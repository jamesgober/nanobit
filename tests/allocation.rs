//! Decoding hostile input must not allocate more than the limits allow.
//!
//! This file holds a single test because it installs a global allocator
//! that records the largest allocation made by the whole process.

#![cfg(feature = "compression")]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Largest;

static LARGEST: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Largest {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LARGEST.fetch_max(layout.size(), Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        LARGEST.fetch_max(layout.size(), Ordering::Relaxed);
        System.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        LARGEST.fetch_max(new_size, Ordering::Relaxed);
        System.realloc(ptr, layout, new_size)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static ALLOCATOR: Largest = Largest;

#[test]
fn size_prefixes_do_not_drive_allocation() {
    let inputs: Vec<Vec<u8>> = vec![
        // LZ4: claims 512 MiB.
        vec![0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x01],
        // LZ4: claims 4 GiB.
        vec![0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x01],
        // Snappy: claims 4 GiB.
        vec![0xff, 0xff, 0xff, 0xff, 0x0f, 0x00, 0x00, 0x00],
    ];
    for input in &inputs {
        LARGEST.store(0, Ordering::Relaxed);
        assert!(nanobit::decompress(input).is_err());
        let largest = LARGEST.load(Ordering::Relaxed);
        assert!(
            largest < 1 << 20,
            "{input:02x?} made a {largest} byte allocation"
        );
    }
}
