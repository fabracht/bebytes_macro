# BeBytes Crate Reference Guide

## Overview

BeBytes is a high-performance Rust derive macro for binary serialization/deserialization. It generates methods to convert structs to/from byte arrays with support for both big-endian and little-endian byte orders.

## Installation

```toml
[dependencies]
bebytes = "2.8.0"
```

## Basic Usage

```rust
use bebytes::BeBytes;

#[derive(BeBytes)]
struct MyStruct {
    field1: u32,
    field2: u16,
}
```

## Generated Methods

Every struct with `#[derive(BeBytes)]` gets these methods:

### Core Methods
- `field_size() -> usize` - Calculate total size in bytes
- `try_from_be_bytes(&[u8]) -> Result<(Self, usize), BeBytesError>` - Parse from big-endian
- `try_from_le_bytes(&[u8]) -> Result<(Self, usize), BeBytesError>` - Parse from little-endian
- `to_be_bytes(&self) -> Vec<u8>` - Convert to big-endian bytes
- `to_le_bytes(&self) -> Vec<u8>` - Convert to little-endian bytes

### Buffer Methods
- `to_be_bytes_buf(&self) -> Bytes` - Convert to big-endian Bytes buffer (internal implementation, no external deps)
- `to_le_bytes_buf(&self) -> Bytes` - Convert to little-endian Bytes buffer (internal implementation, no external deps)
- `encode_be_to<B: BufMut>(&self, buf: &mut B) -> Result<(), BeBytesError>` - Write to buffer (BE)
- `encode_le_to<B: BufMut>(&self, buf: &mut B) -> Result<(), BeBytesError>` - Write to buffer (LE)

## Field Attributes

### Bit Fields: `#[bits(N)]`
Define fields that use less than 8 bits. Bit fields MUST complete a full byte (sum to multiple of 8).

```rust
#[derive(BeBytes)]
struct Flags {
    #[bits(1)]
    is_active: u8,    // 1 bit
    #[bits(3)]
    priority: u8,     // 3 bits
    #[bits(4)]
    category: u8,     // 4 bits
    // Total: 8 bits = 1 byte ✓
}
```

### Fixed-Size Fields: `#[With(size(N))]`
For strings and vectors with fixed size:

```rust
#[derive(BeBytes)]
struct FixedPacket {
    #[With(size(16))]
    username: String,     // Exactly 16 bytes
    #[With(size(32))]
    data: Vec<u8>,       // Exactly 32 bytes
}
```

### Variable-Size Fields: `#[FromField(field_name)]`
Size determined by another field:

```rust
#[derive(BeBytes)]
struct VarPacket {
    data_len: u16,
    #[FromField(data_len)]
    data: Vec<u8>,       // Size from data_len field
}
```

### Size Expressions: `#[With(size(expression))]`
Mathematical expressions for dynamic sizing:

```rust
#[derive(BeBytes)]
struct Matrix {
    width: u8,
    height: u8,
    #[With(size(width * height))]
    pixels: Vec<u8>,     // Size = width × height
}
```

### Nested Field Access
Access fields in nested structs using dot notation:

```rust
#[derive(BeBytes)]
struct Header {
    version: u8,
    payload_size: u16,
}

#[derive(BeBytes)]
struct Packet {
    header: Header,
    #[FromField(header.payload_size)]
    payload: Vec<u8>,    // Size from nested field
}
```

## Supported Types

### Primitive Types
- **Unsigned integers**: u8, u16, u32, u64, u128
- **Signed integers**: i8, i16, i32, i64, i128
- **Characters**: char (4-byte Unicode)
- **NO SUPPORT**: f32, f64, usize, isize

### Collections
- **Arrays**: `[T; N]` where T is a supported type
- **Vectors**: `Vec<u8>` (must be last field unless size-constrained)
- **Strings**: `String` (same rules as Vec<u8>)

### Enums
Two types of enums are supported:

#### Standard Enums
Must have explicit discriminant values:

```rust
#[derive(BeBytes)]
enum MessageType {
    Request = 1,
    Response = 2,
    Error = 3,
}
```

#### Flag Enums: `#[bebytes(flags)]`
For bitwise operations, all values must be powers of 2:

```rust
#[derive(BeBytes, Copy, Clone)]
#[bebytes(flags)]
enum Permissions {
    None = 0,
    Read = 1,
    Write = 2,
    Execute = 4,
    Admin = 8,
}

// Usage
let perms = Permissions::Read | Permissions::Write;  // = 3
```

### Options
Supported for primitive types only:

```rust
#[derive(BeBytes)]
struct OptionalFields {
    required: u32,
    optional: Option<u16>,  // None serializes as zeros
}
```

## Important Rules

### Bit Field Rules
1. Bit fields MUST complete full bytes (sum to 8, 16, 24, etc.)
2. Use smallest type that fits (u8 for ≤8 bits, u16 for ≤16 bits, etc.)
3. Fields are packed in declaration order

### Vector/String Rules
1. Unbounded vectors/strings can ONLY be the last field
2. Use `#[With(size(N))]` for fixed-size anywhere in struct
3. Use `#[FromField(field)]` for variable-size anywhere in struct
4. Size fields must come BEFORE the fields that reference them

### Byte Order
- Methods with `_be_` use big-endian byte order
- Methods with `_le_` use little-endian byte order
- NO default byte order - must explicitly choose

## Common Patterns

### Network Protocol Header
```rust
#[derive(BeBytes)]
struct TcpHeader {
    source_port: u16,
    dest_port: u16,
    sequence: u32,
    ack_number: u32,
    #[bits(4)]
    data_offset: u8,
    #[bits(3)]
    reserved: u8,
    #[bits(9)]
    flags: u16,
    window_size: u16,
    checksum: u16,
    urgent_pointer: u16,
}
```

### Variable-Length Message
```rust
#[derive(BeBytes)]
struct Message {
    msg_type: u8,
    msg_id: u32,
    payload_len: u16,
    #[FromField(payload_len)]
    payload: Vec<u8>,
}
```

### Fixed-Size String Fields
```rust
#[derive(BeBytes)]
struct UserRecord {
    user_id: u32,
    #[With(size(32))]
    username: String,    // Must be exactly 32 bytes
    #[With(size(64))]
    email: String,       // Must be exactly 64 bytes
    last_login: u64,
}
```

## Error Handling

All `try_from_*_bytes` methods return `Result<(Self, usize), BeBytesError>` where:
- Success: Returns parsed struct and number of bytes consumed
- Failure: Returns `BeBytesError` with details

Common errors:
- `InsufficientData`: Not enough bytes to parse
- `InvalidUtf8`: String field contains invalid UTF-8
- `InvalidEnumValue`: Enum discriminant not recognized

## Performance Tips

1. Use buffer methods (`encode_be_to`) for direct writing without intermediate allocations
2. Place variable-size fields at the end when possible
3. Use bit fields to pack boolean flags efficiently
4. The internal buffer types (`Bytes`, `BytesMut`) provide efficient memory management without external dependencies

## DO NOT

- **DO NOT** use `repr(u8)` or other repr attributes on enums - bebytes handles this automatically
- **DO NOT** use f32/f64 - floating point types are not supported
- **DO NOT** use usize/isize - platform-dependent sizes are not supported
- **DO NOT** place unbounded Vec/String fields anywhere except last position
- **DO NOT** reference fields that come after the current field in size expressions

## Example: Complete Network Packet

```rust
use bebytes::BeBytes;

#[derive(BeBytes, Debug)]
struct NetworkPacket {
    // Bit-packed header flags
    #[bits(1)]
    is_encrypted: u8,
    #[bits(1)]
    is_compressed: u8,
    #[bits(2)]
    priority: u8,
    #[bits(4)]
    version: u8,
    
    // Standard fields
    packet_id: u32,
    timestamp: u64,
    
    // Size fields for variable data
    sender_len: u8,
    payload_len: u16,
    
    // Variable-size fields
    #[FromField(sender_len)]
    sender_name: String,
    #[FromField(payload_len)]
    payload: Vec<u8>,
}

fn main() {
    let packet = NetworkPacket {
        is_encrypted: 1,
        is_compressed: 0,
        priority: 2,
        version: 4,
        packet_id: 0x12345678,
        timestamp: 1234567890,
        sender_len: 5,
        payload_len: 11,
        sender_name: "Alice".to_string(),
        payload: b"Hello World".to_vec(),
    };
    
    // Serialize
    let bytes = packet.to_be_bytes();
    
    // Deserialize
    let (parsed, bytes_read) = NetworkPacket::try_from_be_bytes(&bytes).unwrap();
    
    println!("Packet size: {} bytes", bytes_read);
}
```