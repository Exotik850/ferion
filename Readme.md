# Ferion

### Lightning fast serialization and deserialization for the [Rion Data format](https://jenkov.com/tutorials/rion/index.html)

Ferion is a high-performance Rust library for working with RION (Raw Internet Object Notation), a compact and efficient binary serialization format. RION combines the flexibility of JSON with the compactness of binary formats, making it ideal for high-throughput data processing and storage applications.

## Features

- **Blazing Fast**: Optimized for speed, Ferion outperforms JSON serialization in both encoding and decoding.
- **Memory Efficient**: RION's compact binary format significantly reduces memory usage compared to text-based formats.
- **Seamless Integration**: Works smoothly with Serde, allowing easy serialization of Rust structs and enums.
- **Type Safety**: Leverages Rust's type system to ensure correctness at compile-time.
- **Zero-Copy Deserialization**: Supports borrowing data directly from the input buffer for maximum efficiency.

## Installation

Add Ferion to your `Cargo.toml`:

```toml
[dependencies]
ferion = "0.1.0"
```

## Quick Start

```rust
use ferion::{to_bytes, from_bytes};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Person {
    name: String,
    age: u32,
}

fn main() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    // Serialize to RION
    let encoded: Vec<u8> = to_bytes(&person).unwrap();

    // Deserialize from RION
    let decoded: Person = from_bytes(&encoded).unwrap();

    assert_eq!(person, decoded);
    println!("Serialization and deserialization successful!");
}
```

## Contributing

We welcome contributions to Ferion! Here's how you can help:

1. Fork the repository
2. Create a new branch (`git checkout -b feature/amazing-feature`)
3. Make your changes and commit (`git commit -am 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

Ferion is licensed under the MIT License, see [License](./License.md) for details