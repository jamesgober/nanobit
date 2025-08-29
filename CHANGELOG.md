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
[Unreleased]: https://github.com/jamesgober/rust-benchmark/compare/v0.2.0...HEAD
[0.3.0]: https://github.com/jamesgober/rust-benchmark/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/jamesgober/rust-benchmark/releases/tag/v0.2.0
[0.1.0]: #
