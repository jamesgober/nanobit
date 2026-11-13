<h1 align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br>
    <b>CHANGELOG</b>
</h1>
<p>
  All notable changes to this project will be documented in this file. The format is based on <a href="https://keepachangelog.com/en/1.1.0/">Keep a Changelog</a>,
  and this project adheres to <a href="https://semver.org/spec/v2.0.0.html/">Semantic Versioning</a>.
</p>

## [Unreleased]

<br>

## [0.3.0] - 2026-11-13

This release makes decoding safe on input you do not control. Upgrading is
recommended for anyone who decodes or decompresses data from outside the
process.

### Fixed
- A string or byte slice whose length prefix was close to `usize::MAX` made
  `from_bytes` panic. Lengths are now checked without overflow and such
  input returns `Error::UnexpectedEof`.
- `decompress` allocated whatever size an LZ4 or Snappy header claimed (up to
  4 GiB) before checking anything. The claimed size is now checked against a
  limit and against what the input could expand to before any allocation.
- ZSTD output had no ceiling. It now stops at the same limit.
- Deeply nested input could overflow the stack and abort the process.
  Nesting is now limited (128 levels by default).
- Varints that do not fit in a `u64`, and lengths that do not fit in `usize`
  on 32-bit targets, are rejected instead of being truncated.
- `cargo build --no-default-features` failed. The crate now builds for
  `no_std` targets with `alloc`.
- The `async` feature pointed at modules that were never in the crate, so it
  never compiled (#3).

### Added
- `decompress_with_limit` and `deserialize_compressed_with_limit` to set the
  most bytes decompression may produce.
- `DEFAULT_MAX_DECOMPRESSED_SIZE` (64 MiB), used by `decompress` and
  `deserialize_compressed`.
- `Deserializer::with_max_depth`, `Deserializer::max_depth` and
  `DEFAULT_MAX_DEPTH` (128).
- `ReadBuffer::read_len` for reading a length or count.
- `from_reader_with_limit`.

### Changed
- `decompress` and `deserialize_compressed` now refuse output larger than
  64 MiB. Use the `_with_limit` versions for larger data.
- serde is now a required dependency; it was already needed to build. The
  `serde` and `serde_compat` features are kept and do nothing.
- The `compression` and `multi-compression` features now enable `std`.
- `to_writer`, `from_reader` and `from_reader_with_limit` are only exported
  with the `std` feature, which they always needed.

### Removed
- The `async` feature and its optional `tokio`, `futures-core` and
  `futures-util` dependencies.

<br>

## [0.2.0] - 2025-08-29

### Added
- Multi-format compression support with CompressionFormat enum (LZ4, ZSTD, Snappy)
- CompressionLevel enum for configurable compression levels
- compress() function with format and level parameters
- compress_default() for quick compression with defaults
- decompress() function with automatic format detection
- is_serialized() function to check if data is nanobit format
- Enhanced compression module with comprehensive error handling
- Support for future custom NanoBit compression format
- Multi-compression feature flag for optional compression algorithms

### Changed
- Moved compression functionality to dedicated compression.rs module
- Enhanced compression API for better flexibility and performance
- Added zstd and snap dependencies for additional compression formats

### Fixed
- Resolved duplicate compression exports in lib.rs
- Added proper feature flags for optional compression dependencies

<br>

## [0.1.0] - 2025-08-29
### Added
- Initial release with high-performance binary serialization
- Custom binary format with NANO magic bytes and version 1
- Zero-copy deserialization support for strings and byte slices
- Serde compatibility for seamless integration
- Async serialization support with tokio integration
- LZ4 compression support with configurable levels
- Thread-safe operations for concurrent usage
- Varint encoding for space efficiency
- Custom WriteBuffer and ReadBuffer for optimized I/O
- Comprehensive examples (basic_usage.rs)
- Performance benchmarks with criterion
- to_bytes() and from_bytes() API for easy usage
- serialize() and deserialize() convenience functions
- Optional compression with serialize_compressed()/deserialize_compressed()

### Changed
- N/A

### Fixed
- Removed redundant explicit outlives requirements (`'de: 'a`) from SeqDeserializer, MapDeserializer, and EnumDeserializer structs
- Fixed rustc warnings about lifetime bound inference

### Removed
- N/A


<!-- FOOT LINKS
################################################# -->
[Unreleased]: https://github.com/jamesgober/nanobit/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/jamesgober/nanobit/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/jamesgober/nanobit/releases/tag/v0.2.0
[0.1.0]: #
