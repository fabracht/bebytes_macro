# BeBytes

BeBytes is a trait wrapper around the BeBytes derive crate.

## BeBytes Derive

Derive is a procedural macro crate that provides a custom derive macro for generating serialization and deserialization methods for network structs in Rust. The macro generates code to convert the struct into a byte representation (serialization) and vice versa (deserialization) supporting both big endian and little endian byte orders. It aims to simplify the process of working with network protocols and message formats by automating the conversion between Rust structs and byte arrays.

For more information, see the [BeBytes Derive crate](https://crates.io/crates/bebytes_derive).

## Usage

To use BeBytes, add it as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
bebytes = "2.3.0"
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

BeBytes provides fine-grained control over bit fields through the `bits` attribute:

```rust
#[derive(BeBytes, Debug)]
struct MyStruct {
    #[bits(1)]
    field1: u8,   // 1 bit
    #[bits(4)]
    field2: u8,   // 4 bits
    #[bits(3)]
    field3: u8,   // 3 bits (total: 8 bits = 1 byte)
    field4: u32,  // Regular 4-byte field
}
```

The `bits` attribute takes a single parameter:

- `bits(n)`: The number of bits this field uses

Key points:

- Bit positions are automatically calculated based on field order
- Bits fields MUST complete a full byte before any non-bits field
- The sum of all bits within a group must equal 8 (or a multiple of 8)

### Multi-Byte Bit Fields

BeBytes supports bit manipulation on all integer types from `u8`/`i8` to `u128`/`i128`:

```rust
#[derive(BeBytes, Debug)]
struct U16Example {
    #[bits(1)]
    flag: u8,     // 1 bit
    #[bits(14)]
    value: u16,   // 14 bits spanning across bytes
    #[bits(1)]
    last_flag: u8,  // 1 bit (total: 16 bits = 2 bytes)
}
```

The same rules apply - all bits fields must complete a byte boundary together.

### Enum Bit Packing

Enums can be used with the `#[bits()]` attribute for automatic bit-width calculation. While `#[repr(u8)]` is not strictly required, it is recommended as it makes the u8 constraint explicit and provides compile-time guarantees:

```rust
#[derive(BeBytes, Debug, PartialEq)]
#[repr(u8)]  // Recommended: ensures discriminants fit in u8 at compile time
enum Status {
    Idle = 0,
    Running = 1,
    Paused = 2,
    Stopped = 3,
}

#[derive(BeBytes)]
struct PacketHeader {
    #[bits(4)]
    version: u8,
    #[bits()]  // Automatically uses 2 bits (minimum for 4 variants)
    status: Status,
    #[bits(2)]
    flags: u8,
}
```

Key features:

- Automatic bit calculation: `ceil(log2(max_discriminant + 1))`
- No need to specify the bit width in both enum definition and usage
- Type-safe conversion with generated `TryFrom<u8>` implementation
- Supports byte-spanning fields automatically
- Compile-time validation: discriminants exceeding u8 range (255) will produce an error
- Works without `#[repr(u8)]`, but using it is recommended for clarity and compile-time safety

### Flag Enums

BeBytes supports flag-style Enums marked with `#[bebytes(flags)]`. These Enums automatically implement bitwise operations (`|`, `&`, `^`, `!`) allowing them to be used as bit flags:

```rust
#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[bebytes(flags)]
#[repr(u8)]
enum Permissions {
    None = 0,
    Read = 1,
    Write = 2,
    Execute = 4,
    Delete = 8,
}

// Usage
let read_write = Permissions::Read | Permissions::Write;  // = 3
let all_perms = Permissions::Read | Permissions::Write | Permissions::Execute | Permissions::Delete;  // = 15

// Check if a flag is set
assert!(Permissions::Read.contains(Permissions::Read));
assert!(!Permissions::Read.contains(Permissions::Write));

// Toggle flags
let perms = Permissions::Read | Permissions::Execute;
let toggled = perms ^ Permissions::Execute as u8;  // Removes Execute

// Validate flag combinations
assert_eq!(Permissions::from_bits(7), Some(7));  // Valid: Read|Write|Execute
assert_eq!(Permissions::from_bits(16), None);    // Invalid: 16 is not a valid flag
```

Key features:

- All Enum variants must have power-of-2 values (1, 2, 4, 8, etc.)
- Zero value is allowed for "None" or empty flags
- Automatic implementation of bitwise operators
- `contains()` method to check if a flag is set
- `from_bits()` method to validate flag combinations

## Supported Types

BeBytes supports:

- Primitives: `u8`, `u16`, `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`
- Characters: `char` with full Unicode support
- Strings: Standard Rust `String` type with attributes for size control
- Arrays: `[u8; N]`, `[u16; N]`, etc.
- Enums with named fields (serialized as a single byte)
- Enums with `#[bits()]` for automatic bit-width calculation
- `Option<T>` where T is a primitive
- Nested structs that also implement `BeBytes`
- `Vec<T>` with some restrictions (see below)

## String Support

BeBytes provides comprehensive support for Rust's standard `String` type with flexible size control:

### 1. Fixed-Size Strings

Use `#[With(size(N))]` for strings that must be exactly N bytes:

```rust
#[derive(BeBytes)]
struct FixedSizeMessage {
    #[With(size(16))]
    username: String,    // Exactly 16 bytes
    #[With(size(64))]
    message: String,     // Exactly 64 bytes
}
```

**Note**: Fixed-size strings must be padded to the exact length by the user.

### 2. Variable-Size Strings

Use `#[FromField(field_name)]` to specify the size from another field:

```rust
#[derive(BeBytes)]
struct VariableSizePacket {
    name_len: u8,
    desc_len: u16,
    #[FromField(name_len)]
    name: String,         // Size comes from name_len field
    #[FromField(desc_len)]
    description: String,  // Size comes from desc_len field
}
```

### 3. Unbounded Strings

A string as the last field will consume all remaining bytes:

```rust
#[derive(BeBytes)]
struct LogMessage {
    timestamp: u64,
    level: u8,
    message: String,  // Consumes all remaining bytes
}
```

### String Features

- **UTF-8 Validation**: All strings are validated during deserialization
- **Standard Types**: Uses Rust's familiar `String` type
- **Memory Safe**: Proper bounds checking and validation
- **No-std Support**: Works in embedded environments (requires `alloc`)

## Character Support

The `char` type is fully supported with proper Unicode validation:

```rust
#[derive(BeBytes)]
struct UnicodeData {
    symbol: char,
    #[bits(16)]  // Chars can be used in bit fields
    compressed_char: char,
}
```

Characters are stored as 4-byte Unicode scalar values with validation to ensure they represent valid Unicode code points.

## Size Expressions (New in 2.3.0)

BeBytes now supports dynamic field sizing using mathematical expressions. This powerful feature enables protocol implementations where field sizes depend on other fields:

```rust
#[derive(BeBytes)]
struct NetworkMessage {
    header_size: u8,
    payload_count: u16,
    
    #[With(size(header_size))]        // Size from field
    header: Vec<u8>,
    
    #[With(size(payload_count * 8))]   // Mathematical expression
    payload: Vec<u8>,
}
```

### Supported Operations

- **Mathematical**: `+`, `-`, `*`, `/`, `%` with parentheses
- **Field References**: Reference any previously defined field
- **Complex Expressions**: `#[With(size((width * height) + padding))]`

### Protocol Examples

```rust
// MQTT Packet
#[derive(BeBytes)]
struct MqttPacket {
    fixed_header: u8,
    remaining_length: u8,
    #[With(size(remaining_length))]
    payload: Vec<u8>,
}

// IPv4 Packet  
#[derive(BeBytes)]
struct Ipv4Packet {
    version: u8,
    header_length: u8,
    // ... other fields ...
    #[With(size(4))]  // IPv4 addresses are 4 bytes
    source_address: Vec<u8>,
    #[With(size(4))]
    dest_address: Vec<u8>,
}
```

Size expressions work with both `Vec<u8>` and `String` fields, enabling dynamic sizing for binary protocols while maintaining compile-time validation of expression syntax.

## Vector Support

Vectors require special handling since their size is dynamic. BeBytes provides several ways to handle vectors:

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

### 3.1 Nested Field Access

You can also reference fields in nested structures using dot notation:

```rust
#[derive(BeBytes, Clone)]
struct Header {
    version: u8,
    count: u16,
}

#[derive(BeBytes)]
struct Packet {
    header: Header,
    #[FromField(header.count)]
    items: Vec<Item>,  // Will read 'header.count' items
}

// Even deeply nested fields are supported:
#[derive(BeBytes, Clone)]
struct ComplexHeader {
    meta: MetaInfo,
}

#[derive(BeBytes, Clone)]
struct MetaInfo {
    item_count: u32,
}

#[derive(BeBytes)]
struct ComplexPacket {
    header: ComplexHeader,
    #[FromField(header.meta.item_count)]
    items: Vec<Item>,  // Will read 'header.meta.item_count' items
}
```

### 4. Vectors of Custom Types

BeBytes supports vectors containing custom types that implement the `BeBytes` trait:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct CustomType {
    id: u16,
    value: u32,
}

#[derive(BeBytes, Debug)]
struct VectorOfCustoms {
    count: u8,
    #[FromField(count)]
    items: Vec<CustomType>,  // Vector of custom structs
}
```

For vectors of custom types, the following rules apply:

- When used as the last field, it will consume all remaining bytes, parsing them as instances of the custom type
- When used elsewhere, you must specify size information with `#[FromField]` or `#[With]`
- Each item in the vector is serialized/deserialized using its own BeBytes implementation

## No-STD Support

BeBytes supports no_std environments through feature flags:

```toml
[dependencies]
bebytes = { version = "2.3.0", default-features = false }
```

By default, the `std` feature is enabled. Disable it for no_std support.

## Example: DNS Name Parsing

This example shows how BeBytes can be used to parse a DNS name with dynamic length segments, demonstrating both `#[FromField]` attribute and vectors of custom types:

```rust
#[derive(BeBytes, Debug, Clone, PartialEq)]
struct DnsNameSegment {
    length: u8,
    #[FromField(length)]
    segment: Vec<u8>,  // Dynamic-length byte sequence
}

#[derive(BeBytes, Debug, PartialEq)]
struct DnsName {
    segments: Vec<DnsNameSegment>,  // Vector of custom objects as last field
}

// Usage example
fn main() {
    // Create a DNS name with two segments
    let dns_name = DnsName {
        segments: vec![
            DnsNameSegment {
                length: 3,
                segment: vec![b'w', b'w', b'w'],
            },
            DnsNameSegment {
                length: 7,
                segment: vec![b'e', b'x', b'a', b'm', b'p', b'l', b'e'],
            },
        ],
    };

    // Serialize to bytes
    let bytes = dns_name.to_be_bytes();

    // Deserialize back
    let (reconstructed, _) = DnsName::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(dns_name, reconstructed);
}
```

## Contribute

I'm doing this for fun, but all help is appreciated.

## License

This project is licensed under the [MIT License](https://mit-license.org/)
