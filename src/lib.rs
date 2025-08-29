//! # NanoBit - High-Performance Async Binary Serialization
//!
//! A fast, efficient, and ergonomic binary serialization library with async support.
//!
//! ## Features
//!
//! - **Zero-copy deserialization** where possible
//! - **Async/await support** for non-blocking I/O
//! - **High performance** with minimal allocations
//! - **Flexible compression** with LZ4 support
//! - **Serde compatibility** for easy integration
//! - **No unsafe code** in the core library
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
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec, string::String};

pub mod error;
pub mod ser;
pub mod de;
pub mod buffer;
pub mod compression;

#[cfg(feature = "async")]
pub mod async_ser;
#[cfg(feature = "async")]
pub mod async_de;

// Re-export main types
pub use error::{Error, Result};
pub use ser::{Serializer, to_bytes, to_writer};
pub use de::{Deserializer, from_bytes, from_reader};
pub use buffer::{WriteBuffer, ReadBuffer};

#[cfg(feature = "async")]
pub use async_ser::{AsyncSerializer, to_bytes_async, to_writer_async};
#[cfg(feature = "async")]
pub use async_de::{AsyncDeserializer, from_bytes_async, from_reader_async};

// Enhanced multi-format compression functionality
pub use compression::{
    CompressionFormat, CompressionLevel, 
    compress, decompress, compress_default, is_serialized
};

/// Magic bytes to identify NanoBit format
pub const MAGIC: &[u8] = b"NANO";

/// Current format version
pub const VERSION: u8 = 1;

/// Default buffer size for serialization
pub const DEFAULT_BUFFER_SIZE: usize = 8192;

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
#[cfg(any(feature = "compression", feature = "multi-compression"))]
pub fn deserialize_compressed<T>(bytes: &[u8]) -> Result<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let decompressed = decompress(bytes)?;
    let borrowed_bytes = &decompressed;
    from_bytes(borrowed_bytes)
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
        assert_eq!(42u32, deserialize::<u32>(&serialize(&42u32).unwrap()).unwrap());
        assert_eq!(-100i64, deserialize::<i64>(&serialize(&-100i64).unwrap()).unwrap());
        assert_eq!(3.14f64, deserialize::<f64>(&serialize(&3.14f64).unwrap()).unwrap());
        assert_eq!(true, deserialize::<bool>(&serialize(&true).unwrap()).unwrap());
        assert_eq!("hello", deserialize::<&str>(&serialize(&"hello").unwrap()).unwrap());
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
