# BeBytes Derive

BeBytes Derive is a high-performance procedural macro crate that provides custom derive macros for generating ultra-fast serialization and deserialization methods for network structs in Rust. The macro generates code to convert structs into byte representations and vice versa, supporting both big endian and little endian byte orders.

## 🚀 New in 2.6.0: bytes Crate Integration

BeBytes now uses the `bytes` crate natively for buffer management:

- **2.3x performance improvement** with `Bytes` buffers
- **Zero-copy sharing** via reference-counted buffers
- **Direct integration** with tokio, hyper, tonic, and networking libraries  
- **Standard architecture** using established buffer management patterns

## 🏆 Performance Hierarchy

BeBytes offers multiple performance tiers for different use cases:

1. **Raw Pointer Methods** (2.5.0+): **95-190x speedup** - Zero allocations, compile-time safety
2. **bytes Integration** (2.6.0+): **2.3x speedup** - Zero-copy sharing, async ecosystem compatibility
3. **Standard Methods**: Full compatibility - Works with any struct, comprehensive feature support

The macro supports primitive types, characters, strings (with size attributes), enums, arrays, vectors, and nested structs, making it ideal for working with network protocols, binary formats, and high-performance message serialization.

**Note: BeBytes Derive is currently in development and has not been thoroughly tested in production environments. Use it with caution and ensure proper testing and validation in your specific use case.**

## Usage

To use BeBytes Derive, add it as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
bebytes = "2.6.0"
```

Then, import the BeBytes trait from the bebytes_derive crate and derive it for your struct:

```rust
use bebytes_derive::BeBytes;

#[derive(BeBytes)]
struct MyStruct {
    // Define your struct fields here...
}
```

The BeBytes derive macro will generate the following methods for your struct:

**Core methods:**
- `field_size() -> usize`: Calculate the size (in bytes) of the struct.

**Standard serialization (Vec<u8>):**
- `try_from_be_bytes(&[u8]) -> Result<(Self, usize), BeBytesError>`: Parse from big-endian bytes.
- `to_be_bytes(&self) -> Vec<u8>`: Convert to big-endian bytes.
- `try_from_le_bytes(&[u8]) -> Result<(Self, usize), BeBytesError>`: Parse from little-endian bytes.
- `to_le_bytes(&self) -> Vec<u8>`: Convert to little-endian bytes.

**Buffer methods:**
- `to_be_bytes_buf(&self) -> Bytes`: Convert to big-endian `Bytes` buffer.
- `to_le_bytes_buf(&self) -> Bytes`: Convert to little-endian `Bytes` buffer.
- `encode_be_to<B: BufMut>(&self, buf: &mut B) -> Result<(), BeBytesError>`: Write directly to buffer (big-endian).
- `encode_le_to<B: BufMut>(&self, buf: &mut B) -> Result<(), BeBytesError>`: Write directly to buffer (little-endian).

## Example

Here's an example showcasing the usage of the BeBytes Derive:

```rust
use bebytes::BeBytes;

#[derive(Debug, BeBytes)]
struct NetworkMessage {
    #[bits(1)]
    is_encrypted: u8,
    #[bits(4)]
    message_type: u8,
    #[bits(3)]
    priority: u8,
    sender_id: u32,
    content_len: u8,
    #[With(size(16))]
    sender_name: String,          // Fixed-size string field (16 bytes)
    #[FromField(content_len)]
    content: String,              // Variable-length string field
}

fn main() {
    let message = NetworkMessage {
        is_encrypted: 1,
        message_type: 7,
        priority: 3,
        sender_id: 0x12345678,
        content_len: 11,
        sender_name: "alice           ".to_string(), // Padded to 16 bytes
        content: "Hello, Bob!".to_string(),
    };

    // Big endian serialization
    let be_bytes = message.to_be_bytes();
    println!("Big endian bytes: {:?}", be_bytes);
    
    // Little endian serialization  
    let le_bytes = message.to_le_bytes();
    println!("Little endian bytes: {:?}", le_bytes);
    
    // Deserialize from big endian
    let (be_deserialized, be_bytes_read) = NetworkMessage::try_from_be_bytes(&be_bytes).unwrap();
    println!("Deserialized from BE: {:?}, bytes read: {}", be_deserialized, be_bytes_read);
    
    // Deserialize from little endian
    let (le_deserialized, le_bytes_read) = NetworkMessage::try_from_le_bytes(&le_bytes).unwrap();
    println!("Deserialized from LE: {:?}, bytes read: {}", le_deserialized, le_bytes_read);
    
    // Access string fields
    assert_eq!(be_deserialized.sender_name, "alice           ");
    assert_eq!(be_deserialized.content, "Hello, Bob!");
    
    assert_eq!(le_deserialized.sender_name, "alice           ");
    assert_eq!(le_deserialized.content, "Hello, Bob!");
}
```

In this example, we define a `NetworkMessage` struct that combines bit fields with string fields. The `#[bits]` attribute is used to specify bit-level fields that are packed together. The struct uses standard Rust `String` types with attributes to control their serialization: `#[With(size(16))]` for fixed-size strings and `#[FromField(content_len)]` for variable-length strings where the size comes from another field. The BeBytes derive macro generates the serialization and deserialization methods for the struct, handling both the bit packing and string encoding automatically.

## bytes Crate Integration

BeBytes 2.6.0+ integrates the `bytes` crate for improved buffer management and zero-copy operations:

```rust
use bebytes::{BeBytes, Bytes, BytesMut, BufMut};

#[derive(BeBytes, Debug)]
struct TcpPacket {
    source_port: u16,
    dest_port: u16,
    sequence: u32,
    payload_len: u16,
    #[FromField(payload_len)]
    payload: Vec<u8>,
}

fn main() {
    let packet = TcpPacket {
        source_port: 8080,
        dest_port: 443,
        sequence: 0x12345678,
        payload_len: 13,
        payload: b"Hello, world!".to_vec(),
    };

    // Traditional Vec<u8> approach (still available)
    let vec_bytes = packet.to_be_bytes();
    println!("Vec approach: {} bytes", vec_bytes.len());

    // Bytes buffer
    let bytes_buffer: Bytes = packet.to_be_bytes_buf();
    println!("Bytes buffer: {} bytes", bytes_buffer.len());
    
    // Zero-copy sharing
    let shared_buffer = bytes_buffer.clone(); // Increments reference count
    tokio::spawn(async move {
        // Use shared_buffer in async context...
        send_to_network(shared_buffer).await;
    });

    // Direct buffer writing
    let mut buf = bebytes::BytesMut::with_capacity(packet.field_size());
    packet.encode_be_to(&mut buf).unwrap();
    let final_bytes = buf.freeze(); // Convert to immutable Bytes
    
    println!("✅ All methods produce identical results!");
    assert_eq!(vec_bytes, bytes_buffer.as_ref());
    assert_eq!(vec_bytes, final_bytes.as_ref());
}

// Example integration with networking libraries
async fn send_to_network(data: Bytes) {
    // Works seamlessly with tokio, hyper, tonic, etc.
    // let stream = TcpStream::connect("127.0.0.1:8080").await?;
    // stream.write_all(&data).await?;
}
```

### bytes Integration Benefits

1. **Performance**: 2.3x improvement with `to_be_bytes_buf()` vs `to_be_bytes()`
2. **Zero-copy sharing**: `Bytes` can be shared between tasks without copying data
3. **Memory efficiency**: Reference-counted buffers with automatic cleanup
4. **Ecosystem compatibility**: Works with tokio, hyper, tonic, and async networking
5. **Standard patterns**: Uses established buffer management techniques

### Migration Guide

Existing code continues to work unchanged. To leverage bytes benefits:

```rust
// Before (still works)
let data = packet.to_be_bytes();
send_data(data).await;

// After (2.3x faster, zero-copy sharing)
let data = packet.to_be_bytes_buf();
send_data(data).await; // Same signature, better performance
```

## How it works

The `bits` attribute allows you to define bit-level fields. The attribute takes a single parameter specifying the number of bits the field should occupy. For example, `#[bits(4)]` specifies that the field should take only 4 bits. The position is automatically calculated based on the declaration order of fields. The macro will handle the bit manipulation to ensure correct placement in the resulting byte vector. So a `4` in a field marked with `#[bits(4)]`:

4 => 00000100
Shifted and masked => 0100

Fields are read/written sequentially in Big Endian order and MUST complete a multiple of 8.
This means that fields decorated with the `bits` attribute MUST complete a byte before the next non-bit field is provided. For example, the struct

```rust
#[derive(Debug, BeBytes)]
struct WrongStruct {
    #[bits(1)]
    field1: u8,
    #[bits(4)]
    field2: u8,
    field3: f32,
}
```

will through a compile time error saying that a `U8` attribute must add up to the full byte.

As long as you follow the above rule, you can create custom sequence of bits by using Rust unsigned integers as types and the derived implementation will take care of the nasty shifting and masking for you.
One of the advantages is that we don't need an intermediate vector implementation to parse groups of or individual bits.

## Multi Byte values

The macro has support for all unsigned types from u8 to u128, as well as signed integers (i8 to i128) and the `char` type for Unicode characters. These can be used in the same way the u8 type is used:

- Using a u16

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct U16 {
    #[bits(1)]
    first: u8,
    #[bits(14)]
    second: u16,
    #[bits(1)]
    fourth: u8,
}
```

- Using a u32

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct U32 {
    #[bits(1)]
    first: u8,
    #[bits(30)]
    second: u32,
    #[bits(1)]
    fourth: u8,
}
```

And so on.

**The same rules apply here. Your `U8` fields must complete a byte, even if they span over multiple bytes.**

## Characters and Strings

BeBytes provides comprehensive support for character and string types, making it easy to work with text data in binary protocols.

### Character Support

The `char` type is fully supported with proper Unicode validation:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct UnicodeMessage {
    symbol: char,
    emoji: char,
    #[bits(16)]  // Chars can also be used in bit fields
    compressed_char: char,
}

let msg = UnicodeMessage {
    symbol: '€',
    emoji: '🦀',
    compressed_char: 'A',  // Fits in 16 bits
};
```

Characters are always stored as 4-byte Unicode scalar values with proper validation to ensure they represent valid Unicode code points.

### String Support

BeBytes uses standard Rust `String` types with attributes to control serialization, similar to how vectors work:

#### 1. Fixed-Size Strings

Use `#[With(size(N))]` for strings that must be exactly N bytes:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct NetworkPacket {
    #[With(size(16))]
    sender: String,    // Exactly 16 bytes
    #[With(size(32))]
    message: String,   // Exactly 32 bytes
}

let packet = NetworkPacket {
    sender: "alice           ".to_string(),  // Must be padded to 16 bytes
    message: "Hello, world!                   ".to_string(), // Padded to 32 bytes
};
```

**Note**: Fixed-size strings must be exactly the specified length. The user is responsible for padding.

#### 2. Variable-Size Strings

Use `#[FromField(field_name)]` to specify the size from another field:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct Message {
    id: u32,
    name_len: u8,
    desc_len: u16,
    #[FromField(name_len)]
    name: String,      // Size comes from name_len field
    #[FromField(desc_len)]  
    description: String,  // Size comes from desc_len field
}

let msg = Message {
    id: 123,
    name_len: 7,
    desc_len: 35,
    name: "user123".to_string(),
    description: "This is a longer message content...".to_string(),
};
```

#### 3. Unbounded Strings (Last Field Only)

If a string is the last field, it can be unbounded and will consume all remaining bytes:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct LogEntry {
    timestamp: u64,
    level: u8,
    message: String,  // Will consume all remaining bytes
}

let entry = LogEntry {
    timestamp: 1640995200,
    level: 3,
    message: "Application started successfully with all modules loaded".to_string(),
};
```

### String Features

- **UTF-8 validation**: All strings are validated during deserialization
- **Standard Rust types**: Uses familiar `String` type, no custom types needed
- **Flexible sizing**: Fixed, variable, or unbounded sizes supported
- **No-std compatibility**: Works in embedded environments (requires `alloc`)
- **Memory safety**: Proper bounds checking and validation

### Nested Field Access

The `#[FromField]` attribute supports dot notation for accessing nested fields:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct Header {
    version: u8,
    name_len: u16,
}

#[derive(BeBytes, Debug, PartialEq)]
struct Packet {
    header: Header,
    #[FromField(header.name_len)]
    name: String,  // Size from nested field
    data: Vec<u8>,
}
```

## Size Expressions (New in 2.3.0)

BeBytes now supports dynamic field sizing using mathematical expressions and field references. This powerful feature enables efficient binary protocol implementations where field sizes depend on other fields in the struct.

### Basic Syntax

Use the `#[With(size(expression))]` attribute to specify dynamic field sizes:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct Message {
    count: u8,
    #[With(size(count * 4))]  // Size = count × 4 bytes
    data: Vec<u8>,
}
```

### Supported Operations

Size expressions support mathematical operations, field references, and parentheses:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct ComplexMessage {
    width: u8,
    height: u8,
    padding: u8,
    
    #[With(size(width + height))]           // Addition
    simple_data: Vec<u8>,
    
    #[With(size(width * height))]           // Multiplication  
    image_data: Vec<u8>,
    
    #[With(size((width * height) + padding))] // Complex expression
    padded_data: Vec<u8>,
}
```

### Protocol Examples

Size expressions make it easy to implement real-world protocols:

```rust
// IPv4 Packet Structure
#[derive(BeBytes, Debug, PartialEq)]
struct Ipv4Packet {
    version: u8,
    header_length: u8,
    total_length: u16,
    // ... other fields ...
    #[With(size(4))]  // IPv4 addresses are always 4 bytes
    source_address: Vec<u8>,
    #[With(size(4))]
    dest_address: Vec<u8>,
}

// DNS Message with Variable Sections
#[derive(BeBytes, Debug, PartialEq)]
struct DnsMessage {
    id: u16,
    flags: u16,
    question_count: u16,
    answer_count: u16,
    
    #[With(size(question_count * 5))]  // ~5 bytes per question
    questions: Vec<u8>,
    
    #[With(size(answer_count * 12))]   // ~12 bytes per answer
    answers: Vec<u8>,
}

// MQTT Packet with Remaining Length
#[derive(BeBytes, Debug, PartialEq)]
struct MqttPacket {
    fixed_header: u8,
    remaining_length: u8,
    
    #[With(size(remaining_length))]    // Payload size from header
    payload: Vec<u8>,
}
```

### String Support

Size expressions work with both `Vec<u8>` and `String` fields:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct StringMessage {
    name_length: u8,
    content_length: u16,
    
    #[With(size(name_length))]
    name: String,
    
    #[With(size(content_length))]
    content: String,
}
```

### Key Features

- **Runtime Evaluation**: Expressions are evaluated during serialization/deserialization
- **Compile-time Validation**: Parser validates expression syntax at compile time
- **Field Dependencies**: Reference any previously defined field in the struct
- **Mathematical Operations**: Full support for `+`, `-`, `*`, `/`, `%` with parentheses
- **Memory Safety**: Proper bounds checking and size validation
- **Performance**: No significant overhead compared to manual size calculations

### Important Notes

- Fields referenced in expressions must be defined **before** the fields that use them
- Size mismatches during serialization will panic with descriptive error messages
- Insufficient data during deserialization returns `BeBytesError::InsufficientData`
- Variable-size fields should generally be placed toward the end of structs

For complete documentation and examples, see [SIZE_EXPRESSIONS.md](SIZE_EXPRESSIONS.md).

## Enums

Only enums with named fields are supported and values are read/written as a byte.
Example:

```rust
#[derive(BeBytes, Debug, PartialEq)]
pub enum DummyEnum {
    SetupResponse = 1,
    ServerStart = 2,
    SetupRequest = 3,
}
```

The following code

```rust
    let dummy_enum = DummyEnum::ServerStart;
    let dummy_enum_bytes = dummy_enum.to_be_bytes();
    println!("DummyEnum: {:?}", dummy_enum_bytes);
```

Produces `DummyEnum: [2]`

### Enum Bit Packing

Enums can be used with the `#[bits(N)]` attribute where you specify the exact number of bits needed. You must specify the bit width explicitly based on the number of enum variants:

```rust
#[derive(BeBytes, Debug, PartialEq)]
enum Status {
    Idle = 0,
    Running = 1,
    Paused = 2,
    Stopped = 3,
}

#[derive(BeBytes, Debug, PartialEq)]
struct PacketHeader {
    #[bits(4)]
    version: u8,
    #[bits(2)]  // 2 bits needed for 4 variants (2^2 = 4)
    status: Status,
    #[bits(2)]
    flags: u8,
}
```

In this example:
- The `Status` enum has 4 variants, which requires 2 bits to represent (2^2 = 4)
- You must specify `#[bits(2)]` explicitly for the status field
- The total struct uses 8 bits (4 + 2 + 2), fitting perfectly in 1 byte

#### Bit Calculation for Enums

To determine the number of bits needed for an enum:
- 2 variants need 1 bit (2^1 = 2)
- 3-4 variants need 2 bits (2^2 = 4)
- 5-8 variants need 3 bits (2^3 = 8)
- 9-16 variants need 4 bits (2^4 = 16)
- And so on...

#### Requirements

1. **Explicit Bit Width**: You must specify the exact number of bits with `#[bits(N)]`
2. **u8 Range**: All discriminant values must be within 0-255 range (automatically validated by BeBytes)
3. **Manual Calculation**: Calculate the required bits based on your enum variant count

### Flag Enums

BeBytes now supports flag-style enums marked with `#[bebytes(flags)]`. These enums automatically implement bitwise operations (`|`, `&`, `^`, `!`) allowing them to be used as bit flags.

```rust
#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[bebytes(flags)]
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

#### Requirements for Flag Enums

1. **Power of 2 Values**: All enum variants must have discriminant values that are powers of 2 (1, 2, 4, 8, 16, etc.)
2. **Zero is Allowed**: A variant with value 0 is allowed for "None" or empty flags
3. **u8 Range**: All discriminant values must be within 0-255 range (automatically validated by BeBytes)

#### Generated Methods

For flag enums, the following additional methods are generated:

- **Bitwise Operators**: `BitOr`, `BitAnd`, `BitXor`, `Not` implementations
- **`contains(self, flag: Self) -> bool`**: Check if a specific flag is set
- **`from_bits(bits: u8) -> Option<u8>`**: Validate and create a flag combination from raw bits
- **`decompose(bits: u8) -> Vec<Self>`**: Decompose a u8 value into individual flag variants
- **`iter_flags(bits: u8) -> impl Iterator<Item = Self>`**: Iterate over individual flag variants set in a u8 value

#### Example: Network Protocol Flags

```rust
#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[bebytes(flags)]
enum ProtocolFlags {
    Encrypted = 1,
    Compressed = 2,
    Authenticated = 4,
    KeepAlive = 8,
    Priority = 16,
}

#[derive(BeBytes, Debug, PartialEq)]
struct NetworkPacket {
    #[bits(3)]
    version: u8,
    #[bits(5)]
    reserved: u8,
    flags: u8,  // Store ProtocolFlags combinations
    payload_size: u16,
    #[FromField(payload_size)]
    payload: Vec<u8>,
}

// Create a packet with multiple flags
let packet = NetworkPacket {
    version: 1,
    reserved: 0,
    flags: ProtocolFlags::Encrypted | ProtocolFlags::Authenticated | ProtocolFlags::KeepAlive,
    payload_size: 1024,
    payload: vec![0; 1024],
};

// Serialize and deserialize
let bytes = packet.to_be_bytes();
let (decoded, _) = NetworkPacket::try_from_be_bytes(&bytes).unwrap();

// Check individual flags (traditional method)
let has_encryption = (decoded.flags & ProtocolFlags::Encrypted as u8) != 0;
let has_compression = (decoded.flags & ProtocolFlags::Compressed as u8) != 0;

// Decompose flags into individual variants (new method)
let active_flags = ProtocolFlags::decompose(decoded.flags);
println!("Active flags: {:?}", active_flags);
// Output: [Encrypted, Authenticated, KeepAlive]

// Iterate over active flags efficiently
for flag in ProtocolFlags::iter_flags(decoded.flags) {
    match flag {
        ProtocolFlags::Encrypted => println!("Packet is encrypted"),
        ProtocolFlags::Authenticated => println!("Packet is authenticated"),
        ProtocolFlags::KeepAlive => println!("Keep-alive flag set"),
        ProtocolFlags::Compressed => println!("Packet is compressed"),
        ProtocolFlags::Priority => println!("High priority packet"),
    }
}
```

## Options

Options are supported, as long as the internal type is a primitive
Example:

```rust
#[derive(BeBytes, Debug, PartialEq)]
pub struct Optional {
    pub optional_number: Option<i32>,
}
```

Options are serialized as the internal type. A `None` is serialized as a zero byte.

## Byte arrays and Vectors

You can pass a static array of bytes, since the size is known at compilation time.
Example:

```rust
pub struct DummyStruct {
    pub dummy0: [u8; 2],
    #[bits(1)]
    pub dummy1: u8,
    #[bits(7)]
    pub dummy2: u8,
}
```

**Vectors can ONLY be used as the last field.**

Example:

```rust
#[derive(BeBytes, Debug, PartialEq)]
pub struct ErrorEstimate {
    #[bits(1)]
    pub s_bit: u8,
    #[bits(1)]
    pub z_bit: u8,
    #[bits(6)]
    pub scale: u8,
    pub dummy_struct: DummyStruct,
    pub padding: Vec<u8>,
}
```

Trying to place a vector anywhere else in the sequence produces a compile time error.

## Nested Fields

You can nest structures with the BeBytes trait, but there are some important rules to follow:

### Rules for Safe Nesting

1. **Unbounded Vectors in Nested Structs**
   - Unbounded vectors (a `Vec<T>` without explicit size constraints) should only be used in the last field of a struct
   - When a struct containing unbounded vectors is nested inside another struct, it should only be used as the last field

2. **Safe Vector Usage in Nested Structs**
   - Vectors with explicit size constraints are safe to use anywhere in nested structs
   - Use either `#[With(size(N))]` or `#[FromField(field_name)]` to specify vector sizes

3. **Why This Matters**
   The problem is that the macro cannot determine how many bytes to consume for an unbounded vector. When nested, this causes parsing errors, as shown below:

```rust
    // ❌ PROBLEMATIC: Nested struct with unbounded vector
    #[derive(Debug, PartialEq, Clone, BeBytes)]
    struct WithTailingVec {
        tail: Vec<u8>, // Unbounded vector
    }

    #[derive(Debug, PartialEq, Clone, BeBytes)]
    struct InnocentStruct {
        innocent: u8,
        mid_tail: WithTailingVec, // Nested struct with unbounded vector 
        real_tail: Vec<u8>,
    }
    
    // This will not deserialize correctly
    fn problematic_example() {
        let innocent_struct = InnocentStruct {
            innocent: 1,
            mid_tail: WithTailingVec { tail: vec![2, 3] },
            real_tail: vec![4, 5],
        };
        let innocent_struct_bytes = innocent_struct.to_be_bytes();
        println!("InnocentStruct: {:?}", innocent_struct_bytes);
        let re_innocent_struct = InnocentStruct::try_from_be_bytes(&innocent_struct_bytes)?;
        println!("ReInnocentStruct: {:?}", re_innocent_struct);
        assert_ne!(innocent_struct, re_innocent_struct.0); // This assertion fails
    }
```

Output showing the parsing problem:

```sh
InnocentStruct: [1, 2, 3, 4, 5]
ReInnocentStruct: (InnocentStruct { innocent: 1, mid_tail: WithTailingVec { tail: [2, 3, 4, 5] }, real_tail: [2, 3, 4, 5] }, 1)
```

### Safe Alternative:

```rust
    // ✅ SAFE: Using size constraints for vectors in nested structs
    #[derive(Debug, PartialEq, Clone, BeBytes)]
    struct SafeNestedVec {
        size: u8,
        #[FromField(size)]
        tail: Vec<u8>, // Size constrained vector
    }

    #[derive(Debug, PartialEq, Clone, BeBytes)]
    struct SafeStruct {
        innocent: u8,
        nested: SafeNestedVec, // Safe to nest because vector has size constraint
        more_data: u32,
    }
```

## Documentation

For detailed technical documentation about the macro implementation:

- [Data Flow Documentation](docs/data-flow.md) - Comprehensive diagrams showing how data flows through the BeBytes derive macro
- [Code Generation Examples](docs/code-generation.md) - Concrete examples of generated code for various field types
- [Mutation Testing](docs/mutation-testing.md) - Information about the project's mutation testing strategy

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Quick Start for Contributors

```bash
# Clone the repo
git clone https://github.com/fabracht/bebytes_macro.git
cd bebytes_macro

# Set up pre-commit hooks
./setup-hooks.sh

# Choose option 1 for fast development checks
```

The pre-commit hooks ensure your commits will pass CI by running:
- Formatting checks (`cargo fmt`)
- Linting (`cargo clippy`)
- Build verification
- Test suite

## License

This project is licensed under the [MIT License](https://mit-license.org/)
