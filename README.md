# BeBytes Derive

BeBytes Derive is a procedural macro crate that provides a custom derive macro for generating serialization and deserialization methods for network structs in Rust. The macro generates code to convert the struct into a byte representation (serialization) and vice versa (deserialization) supporting both big endian and little endian byte orders. It aims to simplify the process of working with network protocols and message formats by automating the conversion between Rust structs and byte arrays.

**Note: BeBytes Derive is currently in development and has not been thoroughly tested in production environments. Use it with caution and ensure proper testing and validation in your specific use case.**

## Usage

To use BeBytes Derive, add it as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
bebytes_derive = "0.2"
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

- `new(args...) -> Self`: A constructor method to create a new instance of your struct. Arguments come from the fields of your struct.
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
    #[U8(size(1), pos(0))]
    field1: u8,
    #[U8(size(4), pos(1))]
    field2: u8,
    #[U8(size(3), pos(5))]
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

In this example, we define a struct MyStruct with four fields. The `#[U8]` attribute is used to specify the size and position of the fields for serialization. The BeBytes derive macro generates the serialization and deserialization methods for the struct, allowing us to easily convert it to bytes and back.

## How it works

The `U8` attribute allows you to define 2 attributes, `pos` and `size`. The position attribute defines the position in current byte where the bits should start. For example, a pos(0), size(4) specifies that the field should take only 4 bits and should start at position 0 from left to right. The macro will displace the bits so that they occupy the correct place in the resulting byte vector when `.to_be_bytes()` is used. So a `4` with pos(0) and size(4):

4 => 00000100
Shifted and masked => 0100

Fields are read/written sequentially in Big Endian order and MUST complete a multiple of 8.
This means that fields decorated with the `U8` attribute MUST complete a byte before the next non `U8` byte is provided. For example, the struct

```rust
#[derive(Debug, BeBytes)]
struct WrongStruct {
    #[U8(size(1), pos(0))]
    field1: u8,
    #[U8(size(4), pos(1))]
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
    #[U8(size(1), pos(0))]
    first: u8,
    #[U8(size(14), pos(1))]
    second: u16,
    #[U8(size(1), pos(15))]
    fourth: u8,
}
```

- Using a u32

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct U32 {
    #[U8(size(1), pos(0))]
    first: u8,
    #[U8(size(30), pos(1))]
    second: u32,
    #[U8(size(1), pos(31))]
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
    #[U8(size(1), pos(0))]
    pub dummy1: u8,
    #[U8(size(7), pos(1))]
    pub dummy2: u8,
}
```

**Vectors can ONLY be used as the last field.**

Example:

```rust
#[derive(BeBytes, Debug, PartialEq)]
pub struct ErrorEstimate {
    #[U8(size(1), pos(0))]
    pub s_bit: u8,
    #[U8(size(1), pos(1))]
    pub z_bit: u8,
    #[U8(size(6), pos(2))]
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
