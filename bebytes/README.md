# BeBytes

BeBytes is a trait wrapper around the BeBytes derive crate.

## BeBytes Derive

Derive is a procedural macro crate that provides a custom derive macro for generating serialization and deserialization methods for network structs in Rust. The macro generates code to convert the struct into a byte representation (serialization) and vice versa (deserialization) using big endian order. It aims to simplify the process of working with network protocols and message formats by automating the conversion between Rust structs and byte arrays.

For more information, see the [BeBytes Derive crate](https://crates.io/crates/bebytes_derive).

## Usage

To use BeBytes, add it as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
bebytes = "*"
```

Then, import the BeBytes trait from the bebytes_derive crate and derive it for your struct:

```rust
use bebytes::BeBytes;

#[derive(BeBytes)]
struct Dummy {
    a: u8,
}

fn build_with_bebytes(input: impl BeBytes) -> Vec<u8> {
    input.to_be_bytes()
}

fn build_from_bytes(input: &[u8]) -> Result<(Dummy, usize), Box<dyn std::error::Error>> {
    Dummy::try_from_be_bytes(input)
}
```

The BeBytes derive macro will generate the following methods for your struct:

- `try_from_be_bytes(&[u8]) -> Result<(Self, usize), Box<dyn std::error::Error>>`: A method to convert a byte slice into an instance of your struct. It returns a Result containing the deserialized struct and the number of consumed bytes.
- `to_be_bytes(&self) -> Vec<u8>`: A method to convert the struct into a byte representation. It returns a `Vec<u8>` containing the serialized bytes.
- `field_size() -> usize`: A method to calculate the size (in bytes) of the struct.

## Contribute

I'm doing this for fun, but all help is appreciated.

## License

This project is licensed under the [MIT License](https://mit-license.org/)
