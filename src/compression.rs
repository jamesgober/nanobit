//! Multi-format compression utilities for nanobit
//!
//! Supports multiple compression algorithms for maximum flexibility and performance

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

/// Supported compression formats
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionFormat {
    /// LZ4 fast compression
    #[default]
    LZ4,
    /// ZSTD high-ratio compression  
    ZSTD,
    /// Snappy fast compression
    Snappy,
    /// Future: Custom nanobit compression
    #[allow(dead_code)]
    NanoBit,
}

/// Compression level for algorithms that support it
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionLevel {
    /// Fastest compression with lower compression ratio
    Fastest,
    /// Default balanced compression
    #[default]
    Default,
    /// Best compression ratio (slower)
    Best,
    /// Custom compression level (algorithm-specific)
    Custom(i32),
}

/// Compress data using the specified format
pub fn compress(
    data: &[u8],
    format: CompressionFormat,
    level: CompressionLevel,
) -> Result<Vec<u8>> {
    match format {
        CompressionFormat::LZ4 => compress_lz4(data, level),
        CompressionFormat::ZSTD => compress_zstd(data, level),
        CompressionFormat::Snappy => compress_snappy(data),
        CompressionFormat::NanoBit => {
            // Future: Custom compression algorithm
            Err(Error::Serde(
                "NanoBit compression not yet implemented".to_string(),
            ))
        }
    }
}

/// Compress data using default format and level
pub fn compress_default(data: &[u8]) -> Result<Vec<u8>> {
    compress(
        data,
        CompressionFormat::default(),
        CompressionLevel::default(),
    )
}

/// Decompress data - automatically detects format from header
///
/// The output is limited to
/// [`DEFAULT_MAX_DECOMPRESSED_SIZE`](crate::DEFAULT_MAX_DECOMPRESSED_SIZE)
/// bytes. Use [`decompress_with_limit`] to choose a different limit.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    decompress_with_limit(data, crate::DEFAULT_MAX_DECOMPRESSED_SIZE)
}

/// Decompress data, producing at most `max_output` bytes.
///
/// LZ4 and Snappy data state their decompressed size up front. That size is
/// checked against `max_output`, and against the most the input could
/// expand to, before any output buffer is allocated. Zstandard output is
/// cut off as soon as it passes `max_output`.
///
/// Output that would be larger than `max_output` returns
/// [`Error::Compression`].
pub fn decompress_with_limit(data: &[u8], max_output: usize) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(Error::InvalidFormat("Empty compressed data".to_string()));
    }

    // Try to detect format from magic bytes/header
    if data.len() >= 4 {
        // ZSTD magic number: 0xFD2FB528
        if data[0..4] == [0x28, 0xB5, 0x2F, 0xFD] {
            return decompress_zstd(data, max_output);
        }

        // Snappy detection (stream format has magic bytes)
        if data.len() >= 6 && &data[0..6] == b"sNaPpY" {
            return decompress_snappy(data, max_output);
        }

        // LZ4 detection (simple heuristic) - try last since it's more ambiguous
        if is_likely_lz4(data) {
            return decompress_lz4(data, max_output);
        }
    }

    // Try each format if detection fails. If one of them recognised the data
    // but it was over the limit, report that rather than an unknown format.
    let mut too_large = None;
    let attempts: [Decoder; 3] = [decompress_zstd, decompress_snappy, decompress_lz4];
    for attempt in attempts {
        match attempt(data, max_output) {
            Ok(result) => return Ok(result),
            Err(err @ Error::Compression(_)) => too_large = Some(err),
            Err(_) => {}
        }
    }

    Err(too_large.unwrap_or_else(|| {
        Error::InvalidFormat("Unable to decompress: unknown format".to_string())
    }))
}

type Decoder = fn(&[u8], usize) -> Result<Vec<u8>>;

#[cfg(any(feature = "compression", feature = "multi-compression"))]
fn output_too_large(max_output: usize) -> Error {
    Error::Compression(format!(
        "Decompressed data is larger than the {max_output} byte limit"
    ))
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

/// The most an LZ4 block of `len` bytes can expand to. Each input byte adds
/// at most 255 bytes of output (a match length extension byte).
#[cfg(feature = "compression")]
fn lz4_max_output(len: usize) -> usize {
    len.saturating_mul(255).saturating_add(64)
}

#[cfg(feature = "compression")]
fn decompress_lz4(data: &[u8], max_output: usize) -> Result<Vec<u8>> {
    let (size, block) = lz4_flex::block::uncompressed_size(data)
        .map_err(|e| Error::InvalidFormat(format!("LZ4 decompression failed: {e}")))?;
    if size > lz4_max_output(block.len()) {
        return Err(Error::InvalidFormat(
            "LZ4 size prefix is larger than the data can hold".to_string(),
        ));
    }
    if size > max_output {
        return Err(output_too_large(max_output));
    }
    lz4_flex::block::decompress(block, size)
        .map_err(|e| Error::InvalidFormat(format!("LZ4 decompression failed: {e}")))
}

#[cfg(not(feature = "compression"))]
fn compress_lz4(_data: &[u8], _level: CompressionLevel) -> Result<Vec<u8>> {
    Err(Error::Serde(
        "LZ4 compression not available - enable 'compression' feature".to_string(),
    ))
}

#[cfg(not(feature = "compression"))]
fn decompress_lz4(_data: &[u8], _max_output: usize) -> Result<Vec<u8>> {
    Err(Error::Serde(
        "LZ4 decompression not available - enable 'compression' feature".to_string(),
    ))
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
fn decompress_zstd(data: &[u8], max_output: usize) -> Result<Vec<u8>> {
    use std::io::Read;

    let decoder = zstd::stream::read::Decoder::with_buffer(data)
        .map_err(|e| Error::InvalidFormat(format!("ZSTD decompression failed: {e}")))?;
    // Read one byte past the limit so that output of exactly `max_output`
    // bytes is accepted and anything longer is caught.
    let cap = u64::try_from(max_output)
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    let mut out = Vec::new();
    decoder
        .take(cap)
        .read_to_end(&mut out)
        .map_err(|e| Error::InvalidFormat(format!("ZSTD decompression failed: {e}")))?;
    if out.len() > max_output {
        return Err(output_too_large(max_output));
    }
    Ok(out)
}

#[cfg(not(feature = "multi-compression"))]
fn compress_zstd(_data: &[u8], _level: CompressionLevel) -> Result<Vec<u8>> {
    Err(Error::Serde(
        "ZSTD compression not available - enable 'multi-compression' feature".to_string(),
    ))
}

#[cfg(not(feature = "multi-compression"))]
fn decompress_zstd(_data: &[u8], _max_output: usize) -> Result<Vec<u8>> {
    Err(Error::Serde(
        "ZSTD decompression not available - enable 'multi-compression' feature".to_string(),
    ))
}

// Snappy implementation
#[cfg(feature = "multi-compression")]
fn compress_snappy(data: &[u8]) -> Result<Vec<u8>> {
    snap::raw::Encoder::new()
        .compress_vec(data)
        .map_err(|e| Error::InvalidFormat(format!("Snappy compression failed: {e}")))
}

/// The most a Snappy block of `len` bytes can expand to. The best case is a
/// 3-byte copy of 64 bytes, so 22 times the input is a safe ceiling.
#[cfg(feature = "multi-compression")]
fn snappy_max_output(len: usize) -> usize {
    len.saturating_mul(22).saturating_add(64)
}

#[cfg(feature = "multi-compression")]
fn decompress_snappy(data: &[u8], max_output: usize) -> Result<Vec<u8>> {
    let size = snap::raw::decompress_len(data)
        .map_err(|e| Error::InvalidFormat(format!("Snappy decompression failed: {e}")))?;
    if size > snappy_max_output(data.len()) {
        return Err(Error::InvalidFormat(
            "Snappy length header is larger than the data can hold".to_string(),
        ));
    }
    if size > max_output {
        return Err(output_too_large(max_output));
    }
    snap::raw::Decoder::new()
        .decompress_vec(data)
        .map_err(|e| Error::InvalidFormat(format!("Snappy decompression failed: {e}")))
}

#[cfg(not(feature = "multi-compression"))]
fn compress_snappy(_data: &[u8]) -> Result<Vec<u8>> {
    Err(Error::Serde(
        "Snappy compression not available - enable 'multi-compression' feature".to_string(),
    ))
}

#[cfg(not(feature = "multi-compression"))]
fn decompress_snappy(_data: &[u8], _max_output: usize) -> Result<Vec<u8>> {
    Err(Error::Serde(
        "Snappy decompression not available - enable 'multi-compression' feature".to_string(),
    ))
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
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
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

        let compressed =
            compress(&data, CompressionFormat::LZ4, CompressionLevel::Default).unwrap();
        assert!(compressed.len() < data.len());

        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    #[cfg(feature = "multi-compression")]
    fn test_zstd_compression() {
        let data = b"Hello, world! This is a test string for compression.".repeat(100);

        let compressed =
            compress(&data, CompressionFormat::ZSTD, CompressionLevel::Default).unwrap();
        assert!(compressed.len() < data.len());

        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    #[cfg(feature = "multi-compression")]
    fn test_snappy_compression() {
        let data = b"Hello, world! This is a test string for compression.".repeat(100);

        let compressed =
            compress(&data, CompressionFormat::Snappy, CompressionLevel::Default).unwrap();
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
    #[cfg(feature = "compression")]
    fn lz4_size_prefix_over_the_limit_is_refused() {
        // A 512 MiB size claim followed by a few bytes of "data".
        let mut input = (512u32 << 20).to_le_bytes().to_vec();
        input.extend_from_slice(&[0, 0, 0, 1]);
        assert!(decompress(&input).is_err());

        // A plausible claim that is over the caller's limit.
        let data = vec![7u8; 10_000];
        let packed = compress(&data, CompressionFormat::LZ4, CompressionLevel::Default).unwrap();
        assert!(matches!(
            decompress_with_limit(&packed, 9_999),
            Err(Error::Compression(_))
        ));
        assert_eq!(decompress_with_limit(&packed, 10_000).unwrap(), data);
    }

    #[test]
    #[cfg(feature = "compression")]
    fn lz4_best_case_ratio_fits_the_bound() {
        let data = vec![0u8; 16 << 20];
        let packed = compress(&data, CompressionFormat::LZ4, CompressionLevel::Default).unwrap();
        assert!(data.len() <= lz4_max_output(packed.len() - 4));
        assert_eq!(decompress(&packed).unwrap(), data);
    }

    #[test]
    #[cfg(feature = "multi-compression")]
    fn snappy_length_over_the_limit_is_refused() {
        // A varint length of u32::MAX with almost no data behind it.
        let input = [0xff, 0xff, 0xff, 0xff, 0x0f, 0x00, 0x00, 0x00];
        assert!(decompress_snappy(&input, usize::MAX).is_err());

        let data = vec![9u8; 10_000];
        let packed = compress(&data, CompressionFormat::Snappy, CompressionLevel::Default).unwrap();
        assert!(matches!(
            decompress_with_limit(&packed, 9_999),
            Err(Error::Compression(_))
        ));
        assert_eq!(decompress_with_limit(&packed, 10_000).unwrap(), data);
    }

    #[test]
    #[cfg(feature = "multi-compression")]
    fn snappy_best_case_ratio_fits_the_bound() {
        let data = vec![0u8; 16 << 20];
        let packed = compress(&data, CompressionFormat::Snappy, CompressionLevel::Default).unwrap();
        assert!(data.len() <= snappy_max_output(packed.len()));
        assert_eq!(decompress(&packed).unwrap(), data);
    }

    #[test]
    #[cfg(feature = "multi-compression")]
    fn zstd_output_is_capped() {
        // About 2 KB of zstd that expands to 64 MiB of zeros.
        let data = vec![0u8; 64 << 20];
        let bomb = compress(&data, CompressionFormat::ZSTD, CompressionLevel::Fastest).unwrap();
        assert!(bomb.len() < 64 << 10);

        assert!(matches!(
            decompress_with_limit(&bomb, 1 << 20),
            Err(Error::Compression(_))
        ));
        // Exactly at the limit is fine.
        assert_eq!(
            decompress_with_limit(&bomb, 64 << 20).unwrap().len(),
            64 << 20
        );
        // One byte over the default limit is not.
        let over = vec![0u8; (64 << 20) + 1];
        let bomb = compress(&over, CompressionFormat::ZSTD, CompressionLevel::Fastest).unwrap();
        assert!(matches!(decompress(&bomb), Err(Error::Compression(_))));
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
