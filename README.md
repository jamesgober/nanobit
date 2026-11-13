<div align="center">
   <img width="120px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <h1>
        <strong>NanoBit</strong>
        <sup>
            <br>
            <sub>BINARY SERIALIZATION + ATOMIC COMPRESSION</sub>
            <br>
        </sup>
    </h1>
        <a href="https://crates.io/crates/nanobit" alt="nanobit on Crates.io"><img alt="Crates.io" src="https://img.shields.io/crates/v/nanobit"></a>
        <span>&nbsp;</span>
        <a href="https://crates.io/crates/nanobit" alt="Download nanobit"><img alt="Crates.io Downloads" src="https://img.shields.io/crates/d/nanobit?color=%230099ff"></a>
        <span>&nbsp;</span>
        <a href="https://docs.rs/nanobit" title="nanobit Documentation"><img alt="docs.rs" src="https://img.shields.io/docsrs/nanobit"></a>
        <span>&nbsp;</span>
        <a href="https://github.com/jamesgober/nanobit/actions"><img alt="GitHub CI" src="https://github.com/jamesgober/nanobit/actions/workflows/ci.yml/badge.svg"></a>
</div>
<br>

> Ultra-fast binary serialization with multi-format compression and zero-copy deserialization

## Perfect

nanobit is a high-performance binary serialization library designed for maximum efficiency and minimal overhead. Built with zero-copy deserialization, multi-format compression support, and seamless serde integration, it's perfect for database storage, network protocols, and high-throughput applications.

## Features

- **🚀 Ultra-Fast Performance** - Zero-copy deserialization where possible with minimal allocations
- **🗜️ Multi-Format Compression** - Built-in support for LZ4, ZSTD, and Snappy compression
- **🔧 Auto-Format Detection** - Intelligent decompression that automatically detects compression formats
- **🎯 no_std Support** - The core format builds without std (it needs `alloc`), for embedded systems
- **📦 Serde Compatible** - Seamless integration with the serde ecosystem
- **🔒 Memory Safe** - No unsafe code in the core library
- **⚙️ Configurable** - Feature flags for optional dependencies and compression algorithms
- **🔄 Future-Ready** - Extensible architecture for custom compression formats

## Installation

Add nanobit to your `Cargo.toml`:

```toml
[dependencies]
nanobit = "0.3"
```

### Feature Flags

Enable specific compression formats:

```toml
[dependencies]
nanobit = { version = "0.3", features = ["compression", "multi-compression"] }
```

Available features:
- `std` - Standard library support: `to_writer`, `from_reader` (enabled by default)
- `compression` - LZ4 compression support, needs `std` (enabled by default)
- `multi-compression` - ZSTD and Snappy support, needs `std` (enabled by default)

serde is always enabled. The old `serde` and `serde_compat` feature names still
work and do nothing.

## Quick Start

```rust
use nanobit::{to_bytes, from_bytes};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user = User {
        id: 12345,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        active: true,
    };

    // Serialize to bytes
    let bytes = to_bytes(&user)?;
    
    // Deserialize from bytes
    let decoded: User = from_bytes(&bytes)?;
    
    assert_eq!(user, decoded);
    println!("Serialized {} bytes", bytes.len());
    
    Ok(())
}
```

## Usage Examples

### Basic Serialization

```rust
use nanobit::{serialize, deserialize};

// Serialize any serde-compatible type
let data = vec![1u32, 2, 3, 4, 5];
let bytes = serialize(&data)?;
let recovered: Vec<u32> = deserialize(&bytes)?;
```

### Compression Support

```rust
use nanobit::{compress, decompress, CompressionFormat, CompressionLevel};

let data = b"Hello, world!".repeat(1000);

// Compress with specific format and level
let compressed = compress(&data, CompressionFormat::ZSTD, CompressionLevel::Best)?;

// Automatic format detection during decompression
let decompressed = decompress(&compressed)?;
```

### Zero-Copy String Deserialization

```rust
use nanobit::{to_bytes, from_bytes};

let message = "Hello, nanobit!";
let serialized = to_bytes(&message)?;

// Zero-copy deserialization - borrows from serialized data
let text: &str = from_bytes(&serialized)?;
println!("Zero-copy: {}", text); // No allocation!
```

### Compression Formats

```rust
use nanobit::{compress, CompressionFormat, CompressionLevel};

let data = b"Compressible data".repeat(100);

// LZ4 - Fastest compression/decompression
let lz4_data = compress(&data, CompressionFormat::LZ4, CompressionLevel::Fastest)?;

// ZSTD - Best compression ratio
let zstd_data = compress(&data, CompressionFormat::ZSTD, CompressionLevel::Best)?;

// Snappy - Balanced speed and compression
let snappy_data = compress(&data, CompressionFormat::Snappy, CompressionLevel::Default)?;
```

### Checking Serialized Data

```rust
use nanobit::{is_serialized, to_bytes};

let data = "test data";
let serialized = to_bytes(&data)?;

if is_serialized(&serialized) {
    println!("Valid nanobit format");
}
```

### Working with Streams

```rust
use nanobit::{to_writer, from_reader};
use std::io::{Cursor, Write};

let data = vec![1, 2, 3, 4, 5];
let mut buffer = Vec::new();

// Serialize to writer
to_writer(&mut buffer, &data)?;

// Deserialize from reader
let cursor = Cursor::new(buffer);
let recovered: Vec<i32> = from_reader(cursor)?;
```

## Performance

nanobit is designed for maximum performance:

- **Zero-copy deserialization** for strings and byte slices
- **Efficient varint encoding** for space optimization
- **Minimal allocations** during serialization/deserialization
- **Fast compression** with multiple algorithm choices
- **Optimized for database storage** with compact binary format

Compression speed and ratio depend on the format: LZ4 is the fastest, ZSTD
gives the best ratio.

## Format Specification

nanobit uses a compact binary format:

```
[MAGIC: 4 bytes]["NANO"] [VERSION: 1 byte] [PAYLOAD: variable]
```

- **Header**: 5 bytes total (magic + version)
- **Magic Bytes**: `"NANO"` for format identification
- **Version**: Currently `0x01` for forward compatibility
- **Payload**: Serde-serialized data with varint encoding

## Error Handling

```rust
use nanobit::{Error, from_bytes};

match from_bytes::<String>(&invalid_data) {
    Ok(data) => println!("Success: {}", data),
    Err(Error::InvalidFormat(msg)) => eprintln!("Invalid format: {}", msg),
    Err(Error::UnsupportedVersion(v)) => eprintln!("Unsupported version: {}", v),
    Err(Error::Compression(msg)) => eprintln!("Compression error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Untrusted Input

Decoding returns an error on malformed input instead of panicking, and the
decoders have limits so a small input cannot use up memory or stack:

- **Nesting**: the deserializer accepts 128 levels of nesting by default
  (`DEFAULT_MAX_DEPTH`). Each `Some`, sequence, map, struct, enum variant and
  newtype struct is one level. Change it with `Deserializer::with_max_depth`.
- **Decompression**: `decompress` and `deserialize_compressed` produce at most
  64 MiB (`DEFAULT_MAX_DECOMPRESSED_SIZE`). LZ4 and Snappy sizes are checked
  before the output buffer is allocated, and ZSTD output stops at the limit.
  Use `decompress_with_limit` or `deserialize_compressed_with_limit` to pick
  your own limit.
- **Readers**: `from_reader` reads the whole stream. Use
  `from_reader_with_limit` for a stream you do not control.

```rust
use nanobit::{decompress_with_limit, Deserializer};
use serde::Deserialize;

fn read_values(compressed: &[u8]) -> Result<Vec<u32>, nanobit::Error> {
    let bytes = decompress_with_limit(compressed, 1024 * 1024)?;
    let mut de = Deserializer::new(&bytes)?.with_max_depth(32);
    Vec::<u32>::deserialize(&mut de)
}
```

## no_std Support

The core format works in `no_std` environments with `alloc`:

```toml
[dependencies]
nanobit = { version = "0.3", default-features = false }
```

Compression needs `std`.

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the Apache-2.0 License - see the [LICENSE](LICENSE) file for details.

---

**Built with ❤️ for high-performance applications**

<!-- COPYRIGHT
############################################# -->
<div align="center">
  <h2></h2>
  <sup>COPYRIGHT <small>&copy;</small> 2025 <strong>JAMES GOBER.</strong></sup>
</div>