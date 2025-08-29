//! Error types for NanoBit serialization

use core::fmt;

#[cfg(feature = "std")]
use std::error::Error as StdError;

/// Result type alias for NanoBit operations
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur during serialization/deserialization
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Invalid or corrupted data format
    InvalidFormat(String),
    
    /// Unexpected end of input while reading
    UnexpectedEof,
    
    /// Buffer overflow during writing
    BufferOverflow,
    
    /// Not enough data to read
    NotEnoughData,
    
    /// Unsupported version
    UnsupportedVersion(u8),
    
    /// Compression/decompression error
    Compression(String),
    
    /// I/O operation failed
    Io(String),
    
    /// Serde serialization error
    Serde(String),
    
    /// Custom error for user-defined problems
    Custom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidFormat(msg) => write!(f, "Invalid format: {msg}"),
            Error::UnexpectedEof => write!(f, "Unexpected end of input"),
            Error::BufferOverflow => write!(f, "Buffer overflow"),
            Error::NotEnoughData => write!(f, "Not enough data to read"),
            Error::UnsupportedVersion(v) => write!(f, "Unsupported version: {v}"),
            Error::Compression(msg) => write!(f, "Compression error: {msg}"),
            Error::Io(msg) => write!(f, "I/O error: {msg}"),
            Error::Serde(msg) => write!(f, "Serialization error: {msg}"),
            Error::Custom(msg) => write!(f, "Error: {msg}"),
        }
    }
}

#[cfg(feature = "std")]
impl StdError for Error {}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Serde(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Serde(msg.to_string())
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

#[cfg(feature = "compression")]
impl From<lz4_flex::block::DecompressError> for Error {
    fn from(err: lz4_flex::block::DecompressError) -> Self {
        Error::Compression(format!("LZ4 decompression failed: {err:?}"))
    }
}

#[cfg(feature = "compression")]
impl From<lz4_flex::block::CompressError> for Error {
    fn from(err: lz4_flex::block::CompressError) -> Self {
        Error::Compression(format!("LZ4 compression failed: {err:?}"))
    }
}
