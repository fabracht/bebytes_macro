# BeBytes Derive

BeBytes Derive is a procedural macro crate that provides a custom derive macro for generating serialization and deserialization methods for network structs in Rust. The macro generates code to convert the struct into a byte representation (serialization) and vice versa (deserialization) supporting both big endian and little endian byte orders. It aims to simplify the process of working with network protocols and message formats by automating the conversion between Rust structs and byte arrays.

**Note: BeBytes Derive is currently in development and has not been thoroughly tested in production environments. Use it with caution and ensure proper testing and validation in your specific use case.**

## Usage

To use BeBytes Derive, add it as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
bebytes = "1.2.0"
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

- `field_size(&self) -> usize`: A method to calculate the size (in bytes) of the struct.

**Big-endian methods:**
- `try_from_be_bytes(&[u8]) -> Result<(Self, usize), Box<dyn std::error::Error>>`: A method to convert a big-endian byte slice into an instance of your struct. It returns a Result containing the deserialized struct and the number of consumed bytes.
- `to_be_bytes(&self) -> Vec<u8>`: A method to convert the struct into a big-endian byte representation. It returns a `Vec<u8>` containing the serialized bytes.

**Little-endian methods:**
- `try_from_le_bytes(&[u8]) -> Result<(Self, usize), Box<dyn std::error::Error>>`: A method to convert a little-endian byte slice into an instance of your struct. It returns a Result containing the deserialized struct and the number of consumed bytes.
- `to_le_bytes(&self) -> Vec<u8>`: A method to convert the struct into a little-endian byte representation. It returns a `Vec<u8>` containing the serialized bytes.

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
        field4: 0x12345678
    };

    // Big endian serialization
    let be_bytes = my_struct.to_be_bytes();
    println!("Big endian bytes: {:?}", be_bytes);
    // Output: [156, 18, 52, 86, 120]
    
    // Little endian serialization
    let le_bytes = my_struct.to_le_bytes();
    println!("Little endian bytes: {:?}", le_bytes);
    // Output: [156, 120, 86, 52, 18]
    
    // Deserialize from big endian
    let (be_deserialized, be_bytes_read) = MyStruct::try_from_be_bytes(&be_bytes).unwrap();
    println!("Deserialized from BE: {:?}, bytes read: {}", be_deserialized, be_bytes_read);
    
    // Deserialize from little endian
    let (le_deserialized, le_bytes_read) = MyStruct::try_from_le_bytes(&le_bytes).unwrap();
    println!("Deserialized from LE: {:?}, bytes read: {}", le_deserialized, le_bytes_read);
    
    // Both should equal the original struct
    assert_eq!(my_struct.field1, be_deserialized.field1);
    assert_eq!(my_struct.field2, be_deserialized.field2);
    assert_eq!(my_struct.field3, be_deserialized.field3);
    assert_eq!(my_struct.field4, be_deserialized.field4);
    
    assert_eq!(my_struct.field1, le_deserialized.field1);
    assert_eq!(my_struct.field2, le_deserialized.field2);
    assert_eq!(my_struct.field3, le_deserialized.field3);
    assert_eq!(my_struct.field4, le_deserialized.field4);
}
```

In this example, we define a struct MyStruct with four fields. The `#[bits]` attribute is used to specify bit-level fields. The position is automatically calculated based on declaration order. The BeBytes derive macro generates the serialization and deserialization methods for the struct, allowing us to easily convert it to bytes and back.

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

The macro has support for all unsigned types from u8 to u128. These can be used in the same way the u8 type is used:

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

Enums can be used with the `#[bits()]` attribute for automatic bit-width calculation. When you use `#[bits()]` (with empty parentheses) on an enum field, the macro automatically calculates the minimum number of bits needed to represent all enum variants. While `#[repr(u8)]` is not strictly required, it is recommended as best practice:

```rust
#[derive(BeBytes, Debug, PartialEq)]
#[repr(u8)]  // Recommended: ensures discriminants fit in u8 at compile time
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
    #[bits()]  // Automatically uses 2 bits (minimum needed for 4 variants)
    status: Status,
    #[bits(2)]
    flags: u8,
}
```

In this example:
- The `Status` enum has 4 variants, which requires 2 bits to represent (2^2 = 4)
- Using `#[bits()]` on the `status` field automatically allocates exactly 2 bits
- The total struct uses 8 bits (4 + 2 + 2), fitting perfectly in 1 byte

#### How It Works

1. **Automatic Bit Calculation**: The macro calculates the minimum bits needed as `ceil(log2(max_discriminant + 1))`
2. **Compile-Time Constant**: Each enum gets a `__BEBYTES_MIN_BITS` constant that can be used at compile time
3. **TryFrom Implementation**: The macro generates a `TryFrom<u8>` implementation for safe conversion from discriminant values

#### Example with Different Enum Sizes

```rust
#[derive(BeBytes, Debug, PartialEq)]
#[repr(u8)]
enum TwoVariants {
    A = 0,
    B = 1,
}  // Needs 1 bit

#[derive(BeBytes, Debug, PartialEq)]
#[repr(u8)]
enum SeventeenVariants {
    V0 = 0, V1 = 1, V2 = 2, /* ... */ V16 = 16,
}  // Needs 5 bits (2^4 = 16 < 17 <= 2^5 = 32)

#[derive(BeBytes, Debug, PartialEq)]
struct MixedPacket {
    #[bits(2)]
    header: u8,
    #[bits()]  // 1 bit
    two_var: TwoVariants,
    #[bits()]  // 5 bits
    seventeen_var: SeventeenVariants,
    // Total: 2 + 1 + 5 = 8 bits = 1 byte
}
```

#### Benefits

1. **No Redundancy**: You don't need to specify bit width in both the enum definition and struct field
2. **Type Safety**: The compiler ensures enum values fit in the allocated bits
3. **Flexibility**: Mix auto-sized enum fields with explicitly-sized integer fields
4. **Efficiency**: Use exactly the bits needed, no more, no less
5. **Safety**: Compile-time validation ensures all discriminants fit within u8 range (0-255)

#### Note on `#[repr(u8)]`

While BeBytes will work without `#[repr(u8)]`:
- Without it: The macro validates at compile time that discriminants fit in u8 range
- With it: The Rust compiler itself enforces the constraint, providing earlier error detection
- **Recommendation**: Always use `#[repr(u8)]` for enums with BeBytes for clarity and safety

### Flag Enums

BeBytes now supports flag-style enums marked with `#[bebytes(flags)]`. These enums automatically implement bitwise operations (`|`, `&`, `^`, `!`) allowing them to be used as bit flags.

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

#### Requirements for Flag Enums

1. **Power of 2 Values**: All enum variants must have discriminant values that are powers of 2 (1, 2, 4, 8, 16, etc.)
2. **Zero is Allowed**: A variant with value 0 is allowed for "None" or empty flags
3. **#[repr(u8)]**: Flag enums must use `#[repr(u8)]` representation

#### Generated Methods

For flag enums, the following additional methods are generated:

- **Bitwise Operators**: `BitOr`, `BitAnd`, `BitXor`, `Not` implementations
- **`contains(self, flag: Self) -> bool`**: Check if a specific flag is set
- **`from_bits(bits: u8) -> Option<u8>`**: Validate and create a flag combination from raw bits

#### Example: Network Protocol Flags

```rust
#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[bebytes(flags)]
#[repr(u8)]
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

// Check individual flags
let has_encryption = (decoded.flags & ProtocolFlags::Encrypted as u8) != 0;
let has_compression = (decoded.flags & ProtocolFlags::Compressed as u8) != 0;
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

## Contribute

I'm doing this for fun, but all help is appreciated.

## License

This project is licensed under the [MIT License](https://mit-license.org/)
