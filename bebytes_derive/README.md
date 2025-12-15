# BeBytes Derive

BeBytes Derive is a procedural macro crate that provides a custom derive macro for generating serialization and deserialization methods for network structs in Rust. The macro generates code to convert the struct into a byte representation (serialization) and vice versa (deserialization) supporting both big endian and little endian byte orders. It aims to simplify the process of working with network protocols and message formats by automating the conversion between Rust structs and byte arrays.

## Usage

To use BeBytes Derive, add it as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
bebytes_derive = "2.12.0"
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

**Big-endian methods:**

- `try_from_be_bytes(&[u8]) -> Result<(Self, usize), BeBytesError>`: Convert a big-endian byte slice into the struct.
- `to_be_bytes(&self) -> Vec<u8>`: Convert the struct into a big-endian byte representation.

**Little-endian methods:**

- `try_from_le_bytes(&[u8]) -> Result<(Self, usize), BeBytesError>`: Convert a little-endian byte slice into the struct.
- `to_le_bytes(&self) -> Vec<u8>`: Convert the struct into a little-endian byte representation.

**Buffer methods:**

- `to_be_bytes_buf(&self) -> Bytes`: Convert to big-endian buffer.
- `to_le_bytes_buf(&self) -> Bytes`: Convert to little-endian buffer.
- `encode_be_to<B: BufMut>(&self, buf: &mut B) -> Result<(), BeBytesError>`: Write directly to a buffer (big-endian).
- `encode_le_to<B: BufMut>(&self, buf: &mut B) -> Result<(), BeBytesError>`: Write directly to a buffer (little-endian).

**Raw pointer methods (New in 2.5.0):**
For eligible structs (no bit fields, ≤256 bytes, primitives/arrays only):

- `supports_raw_pointer_encoding() -> bool`: Check if raw pointer methods are available.
- `RAW_POINTER_SIZE: usize`: Compile-time constant for struct size.
- `encode_be_to_raw_stack(&self) -> [u8; N]`: Stack-allocated encoding (big-endian).
- `encode_le_to_raw_stack(&self) -> [u8; N]`: Stack-allocated encoding (little-endian).
- `unsafe encode_be_to_raw_mut<B: BufMut>(&self, buf: &mut B) -> Result<(), BeBytesError>`: Direct buffer writing (big-endian).
- `unsafe encode_le_to_raw_mut<B: BufMut>(&self, buf: &mut B) -> Result<(), BeBytesError>`: Direct buffer writing (little-endian).

## Example

Here's an example showcasing the usage of the BeBytes Derive:

```rust
use bebytes_derive::BeBytes;

#[derive(Debug, BeBytes)]
struct MyStruct {
    #[bits(1)]
    field1: u8,
    #[bits(4)]
    field2: u8,
    #[bits(3)]
    field3: u8,
    field4: u32,
}

fn main() {
    let my_struct = MyStruct {
        field1: 1,
        field2: 7,
        field3: 12,
        field4: 0
    };

    let bytes = my_struct.to_be_bytes();
    println!("Serialized bytes: {:?}", bytes);

    let deserialized = MyStruct::try_from_be_bytes(&bytes).unwrap();
    println!("Deserialized struct: {:?}", deserialized);
}
```

In this example, we define a struct MyStruct with four fields. The `#[bits]` attribute is used to specify bit-level fields. The BeBytes derive macro generates the serialization and deserialization methods for the struct, allowing us to easily convert it to bytes and back.

## Raw Pointer Methods

BeBytes 2.5.0 introduces raw pointer methods for eligible structs:

```rust
use bebytes_derive::BeBytes;

#[derive(BeBytes)]
struct FastPacket {
    header: u16,
    payload: u64,
    checksum: u32,
}

fn main() {
    let packet = FastPacket {
        header: 0x1234,
        payload: 0xDEADBEEFCAFEBABE,
        checksum: 0xABCDEF12,
    };

    // Check if raw pointer optimization is available
    if FastPacket::supports_raw_pointer_encoding() {
        println!("Struct size: {} bytes", FastPacket::RAW_POINTER_SIZE);

        // Stack-allocated encoding (safe, no heap allocation)
        let bytes = packet.encode_be_to_raw_stack();
        println!("Fast encoding: {:?}", bytes);

        // Compare with standard method for correctness
        let standard_bytes = packet.to_be_bytes();
        assert_eq!(bytes.as_slice(), standard_bytes.as_slice());

        println!("Raw pointer method completed");
    } else {
        println!("Struct not eligible for raw pointer optimization (has bit fields, too large, or contains unsupported types)");
    }
}
```

### Raw Pointer Method Eligibility

Raw pointer methods are generated for structs that meet all criteria:

- **No bit fields** - Structs with `#[bits(N)]` attributes are not eligible
- **Size ≤ 256 bytes** - Larger structs use standard methods
- **Primitive types only** - Supported types:
  - All integers: `u8`, `u16`, `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`
  - Characters: `char`
  - Fixed-size byte arrays: `[u8; N]`

### Safety

- Stack methods are safe - array sizes determined at compile time
- Direct buffer methods include capacity checks
- Methods only generated for eligible structs

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

will throw a compile time error saying that bit fields must add up to a full byte.

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

**The same rules apply here. Your bit fields must complete a byte, even if they span over multiple bytes.**

_The following primitives can be used with the `bits` attribute: u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, char_

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

### Enum Bit Packing (New in 1.2.0)

Enums can now be used with the `#[bits()]` attribute for automatic bit-width calculation. When you use `#[bits()]` (with empty parentheses) on an enum field, the macro automatically calculates the minimum number of bits needed to represent all enum variants.

```rust
#[derive(BeBytes, Debug, PartialEq)]
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
    #[bits()]  // Automatically uses 2 bits (minimum needed for 4 variants)
    status: Status,
    #[bits(2)]
    flags: u8,
}
```

The macro:

- Calculates minimum bits as `ceil(log2(max_discriminant + 1))`
- Generates a `__BEBYTES_MIN_BITS` constant for each enum
- Implements `TryFrom<u8>` for safe conversion from discriminant values
- Handles byte-spanning fields automatically

### Flag Enums

BeBytes supports flag-style enums marked with `#[bebytes(flags)]`. These enums automatically implement bitwise operations (`|`, `&`, `^`, `!`) allowing them to be used as bit flags:

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

// Validate flag combinations
assert_eq!(Permissions::from_bits(7), Some(7));  // Valid: Read|Write|Execute
assert_eq!(Permissions::from_bits(16), None);    // Invalid: 16 is not a valid flag
```

Requirements for flag enums:

- All enum variants must have power-of-2 values (1, 2, 4, 8, etc.)
- Zero value is allowed for "None" or empty flags
- Supports u8, u16, u32, u64, u128 (auto-detected or explicit via `#[bebytes(flags(u32))]`)

## Options

Options are supported, as long as the internal type is a primitive
Example:

```rust
#[derive(BeBytes, Debug, PartialEq)]
pub struct NestedStruct {
    pub dummy_struct: DummyStruct,
    pub optional_number: Option<i32>,
    pub error_estimate: ErrorEstimate,
}
```

## Byte arrays and Vectors

You can pass a static array of bytes, since the size if known at compilation time.
Example:

```rust
#[derive(BeBytes, Debug, PartialEq)]
pub struct DummyStruct {
    pub dummy0: [u8; 2],
    #[bits(1)]
    pub dummy1: u8,
    #[bits(7)]
    pub dummy2: u8,
}
```

Vectors are supported, but you must either provide a hint of how many bytes you are planning to read or ONLY use it as the last field of your struct. This is because the macro needs to know how many bytes it should read from the buffer in order to be able to read the next field properly aligned. If you don't provide a hint, you can still use a vector, as long as it's the last field of the last struct. You can provide a hint by using:

### With(size(#))

With(size(#)) will tell the macro how many bytes to read from the buffer in order to fill the annotated vector.
Example:

```rust
#[derive(BeBytes, Debug, PartialEq)]
pub struct DummyStruct {
    pub dummy0: [u8; 2],
    #[With(size(10))]
    pub vecty: Vec<u8>,
    pub dummy1: u8,
}
```

### FromField(_fieldname_)

FromField(_fieldname_) will tell the macro to use the value of the field with the specified name.
Example:

```rust
#[derive(BeBytes, Debug, PartialEq)]
pub struct ErrorEstimate {
    pub ipv4: u8,
    #[FromField(ipv4)]
    pub address: Vec<u8>,
    pub rest: u8,
}
```

This allows you to read the value as part of the incoming buffer, such as is the case for DNS packets, where the domain name is interleaved by the number that specifies the length of the next part of the name. (3www7example3com)

## Per-Field Endianness

By default, all fields use the endianness of the method called (`to_be_bytes` or `to_le_bytes`). You can override this for individual fields:

```rust
#[derive(BeBytes)]
struct MixedEndianPacket {
    big_field: u32,                    // Uses method's endianness
    #[bebytes(little_endian)]
    little_field: u16,                 // Always little-endian
    #[bebytes(big_endian)]
    explicit_big: u32,                 // Always big-endian
}
```

This is useful for protocols that mix endianness.

## Size Expressions (New in 2.3.0)

BeBytes supports dynamic field sizing using mathematical expressions and field references. This enables binary protocol implementations where field sizes depend on other fields in the struct.

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

```rust
// MQTT Packet with Remaining Length
#[derive(BeBytes, Debug, PartialEq)]
struct MqttPacket {
    fixed_header: u8,
    remaining_length: u8,

    #[With(size(remaining_length))]    // Payload size from header
    payload: Vec<u8>,
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
- **Bounds Checking**: Size validation during serialization/deserialization

### Last field

If you don't provide a hint and try to use a vector in the middle of your struct, the macro will throw a compile time error.

NOTICE: If a vector is used as the last field of another struct, but the struct is not the last field of the parent struct, the macro will read the whole buffer and try to put that as the value of the vector. This is probably not what you want, so just don't do it.

## Characters and Strings

BeBytes supports character and string types for text data in binary protocols.

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
```

Characters are always stored as 4-byte Unicode scalar values with proper validation to ensure they represent valid Unicode code points.

### String Support

BeBytes uses standard Rust `String` types with attributes to control serialization:

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
```

**Note**: Fixed-size strings must be exactly the specified length. The user is responsible for padding.

#### 2. Variable-Size Strings

Use `#[FromField(field_name)]` to specify the size from another field:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct Message {
    name_len: u8,
    desc_len: u16,
    #[FromField(name_len)]
    name: String,      // Size comes from name_len field
    #[FromField(desc_len)]
    description: String,  // Size comes from desc_len field
}
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
```

### String Features

- **UTF-8 validation**: All strings are validated during deserialization
- **Standard Rust types**: Uses familiar `String` type, no custom types needed
- **Flexible sizing**: Fixed, variable, or unbounded sizes supported
- **No-std compatibility**: Works in embedded environments (requires `alloc`)
- **Memory safety**: Proper bounds checking and validation

## Nested Fields

In theory, you can nest structures, but beware of padding vectors. I have not implemented, nor tested anything to prevent you from doing it, so just don't put nested structs with unhinted vectors in it unless they are occupy the last position.

```rust
#[derive(BeBytes, Debug, PartialEq)]
pub struct NestedStruct {
    pub dummy_struct: DummyStruct,
    pub error_estimate: ErrorEstimate,
}
```

## Contribute

I'm doing this for fun, but all help is appreciated.

## License

This project is licensed under the [MIT License](https://mit-license.org/)
