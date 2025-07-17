# BeBytes Derive

BeBytes Derive is a procedural macro crate that provides a custom derive macro for generating serialization and deserialization methods for network structs in Rust. The macro generates code to convert the struct into a byte representation (serialization) and vice versa (deserialization) supporting both big endian and little endian byte orders. It aims to simplify the process of working with network protocols and message formats by automating the conversion between Rust structs and byte arrays.

**Note: BeBytes Derive is currently in development and has not been thoroughly tested in production environments. Use it with caution and ensure proper testing and validation in your specific use case.**

## Usage

To use BeBytes Derive, add it as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
bebytes_derive = "1.1.0"
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
use bebytes_macro::BeBytes;

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

In this example, we define a struct MyStruct with four fields. The `#[U8]` attribute is used to specify the size and position of the fields for serialization. The BeBytes derive macro generates the serialization and deserialization methods for the struct, allowing us to easily convert it to bytes and back.

## How it works

The `bits` attribute allows you to define bit-level fields. The attribute takes a single parameter specifying the number of bits the field should occupy. For example, `#[bits(4)]` specifies that the field should take only 4 bits. The position is automatically calculated based on the declaration order of fields. The macro will handle the bit manipulation to ensure correct placement in the resulting byte vector. So a `4` in a field marked with `#[bits(4)]`:

4 => 00000100
Shifted and masked => 0100

Fields are read/written sequentially in Big Endian order and MUST complete a multiple of 8.
This means that fields decorated with the `U8` attribute MUST complete a byte before the next non `U8` byte is provided. For example, the struct

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

*The following primitives can be used with the `U8` attribute: u8, u16, u32, u64, u128, i8, i16, i32, i64, i128*

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

### From(*fieldname*)

From(*fieldname*) will tell the macro to use the value of the field with the
Example:

```rust
#[derive(BeBytes, Debug, PartialEq)]
pub struct ErrorEstimate {
    pub ipv4: u8,
    #[From(ipv4)]
    pub address: Vec<u8>,
    pub rest: u8,
}
```

This allows you to read the value as part of the incoming buffer, such as is the case for DNS packets, where the domain name is interleaved by the number that specifies the length of the next part of the name. (3www7example3com)

### Last field

If you don't provide a hint and try to use a vector in the middle of your struct, the macro will throw a compile time error.

NOTICE: If a vector is used as the last field of another struct, but the struct is not the last field of the parent struct, the macro will read the whole buffer and try to put that as the value of the vector. This is probably not what you want, so just don't do it.

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
