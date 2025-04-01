# BeBytes

BeBytes is a trait wrapper around the BeBytes derive crate.

## BeBytes Derive

Derive is a procedural macro crate that provides a custom derive macro for generating serialization and deserialization methods for network structs in Rust. The macro generates code to convert the struct into a byte representation (serialization) and vice versa (deserialization) supporting both big endian and little endian byte orders. It aims to simplify the process of working with network protocols and message formats by automating the conversion between Rust structs and byte arrays.

For more information, see the [BeBytes Derive crate](https://crates.io/crates/bebytes_derive).

## Usage

To use BeBytes, add it as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
bebytes = "0.6.1"
```

Then, import the BeBytes trait from the bebytes crate and derive it for your struct:

```rust
use bebytes::BeBytes;

#[derive(BeBytes)]
struct Dummy {
    a: u8,
}

// Using big-endian serialization
fn build_with_be_bytes(input: impl BeBytes) -> Vec<u8> {
    input.to_be_bytes()
}

// Using little-endian serialization
fn build_with_le_bytes(input: impl BeBytes) -> Vec<u8> {
    input.to_le_bytes()
}

// Deserializing from big-endian bytes
fn build_from_be_bytes(input: &[u8]) -> Result<(Dummy, usize), Box<dyn std::error::Error>> {
    Dummy::try_from_be_bytes(input)
}

// Deserializing from little-endian bytes
fn build_from_le_bytes(input: &[u8]) -> Result<(Dummy, usize), Box<dyn std::error::Error>> {
    Dummy::try_from_le_bytes(input)
}
```

## Features

The BeBytes derive macro generates the following methods for your struct:

- `field_size() -> usize`: A method to calculate the size (in bytes) of the struct.

**Big-endian methods:**

- `try_from_be_bytes(&[u8]) -> Result<(Self, usize), Box<dyn std::error::Error>>`: A method to convert a big-endian byte slice into an instance of your struct. It returns a Result containing the deserialized struct and the number of consumed bytes.
- `to_be_bytes(&self) -> Vec<u8>`: A method to convert the struct into a big-endian byte representation. It returns a `Vec<u8>` containing the serialized bytes.

**Little-endian methods:**

- `try_from_le_bytes(&[u8]) -> Result<(Self, usize), Box<dyn std::error::Error>>`: A method to convert a little-endian byte slice into an instance of your struct. It returns a Result containing the deserialized struct and the number of consumed bytes.
- `to_le_bytes(&self) -> Vec<u8>`: A method to convert the struct into a little-endian byte representation. It returns a `Vec<u8>` containing the serialized bytes.

## Bit Field Manipulation

BeBytes provides fine-grained control over bit fields through the `U8` attribute:

```rust
#[derive(BeBytes, Debug)]
struct MyStruct {
    #[U8(size(1), pos(0))]
    field1: u8,   // 1 bit at position 0
    #[U8(size(4), pos(1))]
    field2: u8,   // 4 bits at position 1
    #[U8(size(3), pos(5))]
    field3: u8,   // 3 bits at position 5
    field4: u32,  // Regular 4-byte field
}
```

The `U8` attribute takes two parameters:

- `size(n)`: The number of bits this field uses
- `pos(n)`: The bit position where this field starts (from left to right, 0-indexed)

Fields are read/written sequentially and `U8` fields MUST complete a full byte before the next non-`U8` field. This means the sum of all `size` values within a byte group must be 8 (or a multiple of 8 for multi-byte fields).

### Multi-Byte Bit Fields

BeBytes supports bit manipulation on all unsigned types from `u8` to `u128`:

```rust
#[derive(BeBytes, Debug)]
struct U16Example {
    #[U8(size(1), pos(0))]
    flag: u8,
    #[U8(size(14), pos(1))]
    value: u16,   // 14-bit value spanning across bytes
    #[U8(size(1), pos(15))]
    last_flag: u8,
}
```

The same rules apply - all `U8` fields must complete a byte boundary, even when spanning multiple bytes.

## Supported Types

BeBytes supports:

- Primitives: `u8`, `u16`, `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`
- Arrays: `[u8; N]`, `[u16; N]`, etc.
- Enums with named fields (serialized as a single byte)
- `Option<T>` where T is a primitive
- Nested structs that also implement `BeBytes`
- `Vec<T>` with some restrictions (see below)

## Vector Support

Vectors require special handling since their size is dynamic. BeBytes provides three ways to handle vectors:

### 1. Last Field

A vector can be used as the last field in a struct without additional attributes:

```rust
#[derive(BeBytes)]
struct LastFieldVector {
    header: u32,
    payload: Vec<u8>,  // Will consume all remaining bytes
}
```

### 2. With Size Hint

Use `#[With(size(n))]` to specify the exact number of bytes:

```rust
#[derive(BeBytes)]
struct SizedVector {
    header: u32,
    #[With(size(10))]
    data: Vec<u8>,  // Will read exactly 10 bytes
    footer: u16,
}
```

### 3. From Field

Use `#[FromField(field_name)]` to read the size from another field:

```rust
#[derive(BeBytes)]
struct DynamicVector {
    length: u8,
    #[FromField(length)]
    data: Vec<u8>,  // Will read 'length' bytes
    footer: u16,
}
```

## No-STD Support

BeBytes supports no_std environments through feature flags:

```toml
[dependencies]
bebytes = { version = "0.6.1", default-features = false }
```

By default, the `std` feature is enabled. Disable it for no_std support.

## Example: DNS Name Parsing

This example shows how BeBytes can be used to parse a DNS name with dynamic length segments:

```rust
#[derive(BeBytes, Debug)]
struct DnsNameSegment {
    length: u8,
    #[FromField(length)]
    segment: Vec<u8>,
}

#[derive(BeBytes, Debug)]
struct DnsName {
    segments: Vec<DnsNameSegment>,  // Last field with dynamic segments
}
```

## Contribute

I'm doing this for fun, but all help is appreciated.

## License

This project is licensed under the [MIT License](https://mit-license.org/)
