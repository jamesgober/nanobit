//! Basic usage examples for nanobit serialization

use nanobit::{to_bytes, from_bytes};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct DatabaseRecord {
    id: u64,
    name: String,
    email: String,
    active: bool,
    score: f64,
    tags: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Nanobit String Serialization Examples ===\n");

    // Serialize a simple string
    let message = "Hello, nanobit database!";
    let serialized = to_bytes(&message)?;
    let deserialized: String = from_bytes(&serialized)?;
    println!("String: '{}' -> {} bytes -> '{}'", message, serialized.len(), deserialized);

    // Zero-copy string deserialization (borrowing from serialized data)
    let text: &str = from_bytes(&serialized)?;
    println!("Zero-copy str: '{}' (borrowed, no allocation!)", text);

    // Serialize a database record
    let record = DatabaseRecord {
        id: 12345,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        active: true,
        score: 98.7,
        tags: vec!["admin".to_string(), "premium".to_string(), "verified".to_string()],
    };

    let serialized = to_bytes(&record)?;
    let deserialized: DatabaseRecord = from_bytes(&serialized)?;
    
    println!("\n=== Database Record Serialization ===");
    println!("Original size: ~{} bytes (estimated)", std::mem::size_of_val(&record) + record.name.len() + record.email.len() + record.tags.iter().map(|t| t.len()).sum::<usize>());
    println!("Serialized size: {} bytes", serialized.len());
    println!("Compression ratio: {:.2}%", (serialized.len() as f64 / (std::mem::size_of_val(&record) as f64)) * 100.0);
    println!("Data integrity: {}", if record == deserialized { "✓ PASSED" } else { "✗ FAILED" });

    // Serialize collections efficiently
    let numbers: Vec<u32> = (1..=1000).collect();
    let serialized = to_bytes(&numbers)?;
    let _deserialized: Vec<u32> = from_bytes(&serialized)?;
    
    println!("\n=== Collection Serialization ===");
    println!("Vec<u32> with {} items: {} bytes", numbers.len(), serialized.len());
    println!("Bytes per item: {:.2}", serialized.len() as f64 / numbers.len() as f64);
    
    // Serialize strings of different sizes
    let small_str = "Hi!";
    let medium_str = "This is a medium-length string for testing serialization efficiency";
    let large_str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);
    
    println!("\n=== String Size Analysis ===");
    for (name, s) in [("Small", small_str), ("Medium", medium_str), ("Large", &large_str)] {
        let serialized = to_bytes(&s)?;
        println!("{}: {} chars -> {} bytes (overhead: {} bytes)", 
                 name, s.len(), serialized.len(), serialized.len() - s.len());
    }

    Ok(())
}
