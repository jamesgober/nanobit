//! # NanoBit - High-Performance Binary Serialization
//!
//! A fast, efficient, and ergonomic binary serialization library.
//!
//! ## Features
//!
//! - **Zero-copy deserialization** where possible
//! - **High performance** with minimal allocations
//! - **Flexible compression** with LZ4, Zstandard and Snappy
//! - **Serde compatibility** for easy integration
//! - **No unsafe code** in the core library
//! - **`no_std` support** with `alloc` when the default features are off
//!
//! ## Untrusted input
//!
//! Decoding never panics on malformed input; it returns an [`Error`].
//! Nesting is limited to [`DEFAULT_MAX_DEPTH`] levels (see
//! [`Deserializer::with_max_depth`]), and [`decompress`] refuses to produce
//! more than [`DEFAULT_MAX_DECOMPRESSED_SIZE`] bytes (see
//! [`decompress_with_limit`]).
//!
//! ## Quick Start
//!
//! ```rust
//! use nanobit::{to_bytes, from_bytes};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct Person {
//!     name: String,
//!     age: u32,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let person = Person {
//!     name: "Alice".to_string(),
//!     age: 30,
//! };
//!
//! let bytes = to_bytes(&person)?;
//! let decoded: Person = from_bytes(&bytes)?;
//!
//! assert_eq!(person, decoded);
//! # Ok(())
//! # }
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub mod buffer;
pub mod compression;
pub mod de;
pub mod error;
pub mod ser;

// Re-export main types
pub use buffer::{ReadBuffer, WriteBuffer};
pub use de::{from_bytes, Deserializer};
#[cfg(feature = "std")]
pub use de::{from_reader, from_reader_with_limit};
pub use error::{Error, Result};
pub use ser::to_bytes;
#[cfg(feature = "std")]
pub use ser::to_writer;
pub use ser::Serializer;

// Enhanced multi-format compression functionality
pub use compression::{
    compress, compress_default, decompress, decompress_with_limit, is_serialized,
    CompressionFormat, CompressionLevel,
};

/// Magic bytes to identify NanoBit format
pub const MAGIC: &[u8] = b"NANO";

/// Current format version
pub const VERSION: u8 = 1;

/// Default buffer size for serialization
pub const DEFAULT_BUFFER_SIZE: usize = 8192;

/// How many levels of nesting the deserializer accepts by default.
///
/// Each `Some`, sequence, map, struct, enum variant and newtype struct is one
/// level. Deeper input returns [`Error::InvalidFormat`].
pub const DEFAULT_MAX_DEPTH: usize = 128;

/// The most bytes [`decompress`] will produce: 64 MiB.
///
/// Larger output returns an error before the output buffer is allocated
/// where the format states its size, and as soon as the limit is passed
/// otherwise. Use [`decompress_with_limit`] for a different limit.
pub const DEFAULT_MAX_DECOMPRESSED_SIZE: usize = 64 * 1024 * 1024;

/// Serialize a value to bytes using the default configuration
pub fn serialize<T>(value: &T) -> Result<Vec<u8>>
where
    T: serde::Serialize,
{
    to_bytes(value)
}

/// Deserialize a value from bytes using the default configuration
pub fn deserialize<'de, T>(bytes: &'de [u8]) -> Result<T>
where
    T: serde::Deserialize<'de>,
{
    from_bytes(bytes)
}

/// Serialize with compression
#[cfg(any(feature = "compression", feature = "multi-compression"))]
pub fn serialize_compressed<T>(value: &T, level: CompressionLevel) -> Result<Vec<u8>>
where
    T: serde::Serialize,
{
    let serialized = to_bytes(value)?;
    compress(&serialized, CompressionFormat::default(), level)
}

/// Deserialize compressed data
///
/// The decompressed size is limited to [`DEFAULT_MAX_DECOMPRESSED_SIZE`].
#[cfg(any(feature = "compression", feature = "multi-compression"))]
pub fn deserialize_compressed<T>(bytes: &[u8]) -> Result<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    deserialize_compressed_with_limit(bytes, DEFAULT_MAX_DECOMPRESSED_SIZE)
}

/// Deserialize compressed data whose decompressed size is at most
/// `max_output` bytes.
#[cfg(any(feature = "compression", feature = "multi-compression"))]
pub fn deserialize_compressed_with_limit<T>(bytes: &[u8], max_output: usize) -> Result<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let decompressed = decompress_with_limit(bytes, max_output)?;
    from_bytes(&decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        value: u64,
        items: Vec<i32>,
        flag: bool,
    }

    #[test]
    fn test_basic_serialization() {
        let data = TestStruct {
            name: "test".to_string(),
            value: 42,
            items: vec![1, 2, 3, 4, 5],
            flag: true,
        };

        let serialized = serialize(&data).unwrap();
        let deserialized: TestStruct = deserialize(&serialized).unwrap();

        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_empty_collections() {
        let data = TestStruct {
            name: String::new(),
            value: 0,
            items: vec![],
            flag: false,
        };

        let serialized = serialize(&data).unwrap();
        let deserialized: TestStruct = deserialize(&serialized).unwrap();

        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_primitive_types() {
        assert_eq!(
            42u32,
            deserialize::<u32>(&serialize(&42u32).unwrap()).unwrap()
        );
        assert_eq!(
            -100i64,
            deserialize::<i64>(&serialize(&-100i64).unwrap()).unwrap()
        );
        assert_eq!(
            2.5f64,
            deserialize::<f64>(&serialize(&2.5f64).unwrap()).unwrap()
        );
        assert!(deserialize::<bool>(&serialize(&true).unwrap()).unwrap());
        assert_eq!(
            "hello",
            deserialize::<&str>(&serialize(&"hello").unwrap()).unwrap()
        );
    }

    #[cfg(any(feature = "compression", feature = "multi-compression"))]
    #[test]
    fn test_compression() {
        let data = TestStruct {
            name: "x".repeat(1000), // Compressible data
            value: 12345,
            items: vec![42; 100],
            flag: true,
        };

        let compressed = serialize_compressed(&data, CompressionLevel::Default).unwrap();
        let decompressed: TestStruct = deserialize_compressed(&compressed).unwrap();

        assert_eq!(data, decompressed);

        // Verify compression actually worked for repetitive data
        let uncompressed = serialize(&data).unwrap();
        assert!(compressed.len() < uncompressed.len());
    }
}
