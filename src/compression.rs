//! Multi-format compression utilities for nanobit
//! 
//! Supports multiple compression algorithms for maximum flexibility and performance

use crate::error::{Error, Result};
use serde::{Serialize, Deserialize};

/// Supported compression formats
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionFormat {
    /// LZ4 fast compression
    LZ4,
    /// ZSTD high-ratio compression  
    ZSTD,
    /// Snappy fast compression
    Snappy,
    /// Future: Custom nanobit compression
    #[allow(dead_code)]
    NanoBit,
}

impl Default for CompressionFormat {
    fn default() -> Self {
        Self::LZ4
    }
}

/// Compression level for algorithms that support it
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionLevel {
    /// Fastest compression with lower compression ratio
    Fastest,
    /// Default balanced compression
    Default,
    /// Best compression ratio (slower)
    Best,
    /// Custom compression level (algorithm-specific)
    Custom(i32),
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::Default
    }
}

/// Compress data using the specified format
pub fn compress(data: &[u8], format: CompressionFormat, level: CompressionLevel) -> Result<Vec<u8>> {
    match format {
        CompressionFormat::LZ4 => compress_lz4(data, level),
        CompressionFormat::ZSTD => compress_zstd(data, level),
        CompressionFormat::Snappy => compress_snappy(data),
        CompressionFormat::NanoBit => {
            // Future: Custom compression algorithm
            Err(Error::Serde("NanoBit compression not yet implemented".to_string()))
        }
    }
}

/// Compress data using default format and level
pub fn compress_default(data: &[u8]) -> Result<Vec<u8>> {
    compress(data, CompressionFormat::default(), CompressionLevel::default())
}

/// Decompress data - automatically detects format from header
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(Error::InvalidFormat("Empty compressed data".to_string()));
    }

    // Try to detect format from magic bytes/header
    if data.len() >= 4 {
        // ZSTD magic number: 0xFD2FB528
        if data.len() >= 4 && data[0..4] == [0x28, 0xB5, 0x2F, 0xFD] {
            return decompress_zstd(data);
        }
        
        // Snappy detection (stream format has magic bytes)
        if data.len() >= 6 && &data[0..6] == b"sNaPpY" {
            return decompress_snappy(data);
        }
        
        // LZ4 detection (simple heuristic) - try last since it's more ambiguous
        if is_likely_lz4(data) {
            return decompress_lz4(data);
        }
    }
    
    // Try each format if detection fails, but only if features are enabled
    #[cfg(feature = "multi-compression")]
    if let Ok(result) = decompress_zstd(data) {
        return Ok(result);
    }
    
    #[cfg(feature = "multi-compression")]
    if let Ok(result) = decompress_snappy(data) {
        return Ok(result);
    }
    
    #[cfg(feature = "compression")]
    if let Ok(result) = decompress_lz4(data) {
        return Ok(result);
    }
    
    Err(Error::InvalidFormat("Unable to decompress: unknown format".to_string()))
}

/// Check if data appears to be serialized nanobit format
pub fn is_serialized(data: &[u8]) -> bool {
    if data.len() < 5 {
        return false;
    }
    
    // Check for nanobit magic bytes and valid version
    data.len() >= 5 && &data[0..4] == crate::MAGIC && data[4] == crate::VERSION
}

// LZ4 implementation
#[cfg(feature = "compression")]
fn compress_lz4(data: &[u8], _level: CompressionLevel) -> Result<Vec<u8>> {
    use lz4_flex::compress_prepend_size;
    Ok(compress_prepend_size(data))
}

#[cfg(feature = "compression")]
fn decompress_lz4(data: &[u8]) -> Result<Vec<u8>> {
    use lz4_flex::decompress_size_prepended;
    decompress_size_prepended(data)
        .map_err(|e| Error::InvalidFormat(format!("LZ4 decompression failed: {e}")))
}

#[cfg(not(feature = "compression"))]
fn compress_lz4(_data: &[u8], _level: CompressionLevel) -> Result<Vec<u8>> {
    Err(Error::Serde("LZ4 compression not available - enable 'compression' feature".to_string()))
}

#[cfg(not(feature = "compression"))]
fn decompress_lz4(_data: &[u8]) -> Result<Vec<u8>> {
    Err(Error::Serde("LZ4 decompression not available - enable 'compression' feature".to_string()))
}

// ZSTD implementation
#[cfg(feature = "multi-compression")]
fn compress_zstd(data: &[u8], level: CompressionLevel) -> Result<Vec<u8>> {
    let compression_level = match level {
        CompressionLevel::Fastest => 1,
        CompressionLevel::Default => 3,
        CompressionLevel::Best => 22,
        CompressionLevel::Custom(l) => l,
    };
    
    zstd::encode_all(data, compression_level)
        .map_err(|e| Error::InvalidFormat(format!("ZSTD compression failed: {e}")))
}

#[cfg(feature = "multi-compression")]
fn decompress_zstd(data: &[u8]) -> Result<Vec<u8>> {
    zstd::decode_all(data)
        .map_err(|e| Error::InvalidFormat(format!("ZSTD decompression failed: {e}")))
}

#[cfg(not(feature = "multi-compression"))]
fn compress_zstd(_data: &[u8], _level: CompressionLevel) -> Result<Vec<u8>> {
    Err(Error::Serde("ZSTD compression not available - enable 'multi-compression' feature".to_string()))
}

#[cfg(not(feature = "multi-compression"))]
fn decompress_zstd(_data: &[u8]) -> Result<Vec<u8>> {
    Err(Error::Serde("ZSTD decompression not available - enable 'multi-compression' feature".to_string()))
}

// Snappy implementation
#[cfg(feature = "multi-compression")]
fn compress_snappy(data: &[u8]) -> Result<Vec<u8>> {
    snap::raw::Encoder::new()
        .compress_vec(data)
        .map_err(|e| Error::InvalidFormat(format!("Snappy compression failed: {e}")))
}

#[cfg(feature = "multi-compression")]
fn decompress_snappy(data: &[u8]) -> Result<Vec<u8>> {
    snap::raw::Decoder::new()
        .decompress_vec(data)
        .map_err(|e| Error::InvalidFormat(format!("Snappy decompression failed: {e}")))
}

#[cfg(not(feature = "multi-compression"))]
fn compress_snappy(_data: &[u8]) -> Result<Vec<u8>> {
    Err(Error::Serde("Snappy compression not available - enable 'multi-compression' feature".to_string()))
}

#[cfg(not(feature = "multi-compression"))]
fn decompress_snappy(_data: &[u8]) -> Result<Vec<u8>> {
    Err(Error::Serde("Snappy decompression not available - enable 'multi-compression' feature".to_string()))
}

// LZ4 detection heuristic
fn is_likely_lz4(data: &[u8]) -> bool {
    // Check if it looks like LZ4 with size prefix (lz4_flex format)
    // The first 8 bytes should be the uncompressed size as little-endian
    if data.len() < 8 {
        return false;
    }
    
    // Read the uncompressed size from the first 8 bytes
    let uncompressed_size = u64::from_le_bytes([
        data[0], data[1], data[2], data[3], 
        data[4], data[5], data[6], data[7]
    ]);
    
    // Basic sanity check: uncompressed size should be reasonable
    // (not 0, not ridiculously large compared to compressed size)
    uncompressed_size > 0 && uncompressed_size < (data.len() as u64 * 1000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "compression")]
    fn test_lz4_compression() {
        let data = b"Hello, world! This is a test string for compression.".repeat(100);
        
        let compressed = compress(&data, CompressionFormat::LZ4, CompressionLevel::Default).unwrap();
        assert!(compressed.len() < data.len());
        
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    #[cfg(feature = "multi-compression")]
    fn test_zstd_compression() {
        let data = b"Hello, world! This is a test string for compression.".repeat(100);
        
        let compressed = compress(&data, CompressionFormat::ZSTD, CompressionLevel::Default).unwrap();
        assert!(compressed.len() < data.len());
        
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    #[cfg(feature = "multi-compression")]
    fn test_snappy_compression() {
        let data = b"Hello, world! This is a test string for compression.".repeat(100);
        
        let compressed = compress(&data, CompressionFormat::Snappy, CompressionLevel::Default).unwrap();
        assert!(compressed.len() < data.len());
        
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    #[cfg(feature = "compression")]
    fn test_default_compression() {
        let data = b"Test data for default compression";
        
        let compressed = compress_default(data).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        
        assert_eq!(data, decompressed.as_slice());
    }

    #[test]
    fn test_is_serialized() {
        use crate::{to_bytes, MAGIC};
        
        // Test with actual nanobit data
        let data = "test string";
        let serialized = to_bytes(&data).unwrap();
        assert!(is_serialized(&serialized));
        
        // Test with invalid data
        assert!(!is_serialized(b"invalid"));
        assert!(!is_serialized(&[1, 2, 3]));
        assert!(!is_serialized(&[]));
        
        // Test with correct magic but wrong version
        let mut fake_data = MAGIC.to_vec();
        fake_data.push(99); // wrong version
        fake_data.extend_from_slice(b"data");
        assert!(!is_serialized(&fake_data));
    }

    #[test]
    #[cfg(feature = "multi-compression")]
    fn test_compression_levels() {
        let data = b"Test data".repeat(1000);
        
        let fastest = compress(&data, CompressionFormat::ZSTD, CompressionLevel::Fastest).unwrap();
        let default = compress(&data, CompressionFormat::ZSTD, CompressionLevel::Default).unwrap();
        let best = compress(&data, CompressionFormat::ZSTD, CompressionLevel::Best).unwrap();
        
        // Best compression should be smaller than fastest (usually)
        assert!(best.len() <= default.len());
        
        // All should decompress to original
        assert_eq!(data, decompress(&fastest).unwrap());
        assert_eq!(data, decompress(&default).unwrap());
        assert_eq!(data, decompress(&best).unwrap());
    }
}
