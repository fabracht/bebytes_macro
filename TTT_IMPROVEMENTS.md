# TTT (Type-Trait-Trait) Pattern Improvements

This document describes the TTT pattern improvements made to BeBytes, focusing on enhanced error handling and a type-safe builder pattern.

## Overview

The TTT pattern leverages Rust's type system to create more maintainable and correct code by:
1. **Type**: Creating specific types for domain concepts
2. **Trait (Standard)**: Implementing standard Rust traits (Display, Debug, Error)
3. **Trait (Library)**: Implementing library-specific traits

## Improvements Made

### 1. Enhanced Error Handling

#### Before
The library used a basic error enum with limited context about failures.

#### After
```rust
pub enum BeBytesError {
    EmptyBuffer,
    InsufficientData { expected: usize, actual: usize },
    InvalidDiscriminant { value: u8, type_name: &'static str },
    InvalidBitField { value: u128, max: u128, field: &'static str },
    InvalidUtf8 { field: &'static str },
    MarkerNotFound { marker: u8, field: &'static str },
    ValueOutOfRange { field: &'static str, value: String, max: String },
}
```

#### Benefits
- **Specific error types**: Each error case provides relevant context
- **Better debugging**: Field names and expected values are included
- **Type safety**: Errors are caught at compile time where possible
- **Pattern matching**: Users can handle specific error cases

#### Usage Example
```rust
match NetworkPacket::try_from_be_bytes(&data) {
    Ok((packet, bytes_read)) => {
        // Process packet
    }
    Err(BeBytesError::InsufficientData { expected, actual }) => {
        eprintln!("Need {} more bytes", expected - actual);
    }
    Err(BeBytesError::InvalidUtf8 { field }) => {
        eprintln!("Invalid UTF-8 in field: {}", field);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

### 2. Type-Safe Builder Pattern

#### Purpose
Provides a compile-time safe way to construct byte sequences with proper ordering of fixed and variable-size fields.

#### Type States
```rust
pub struct Empty;       // Initial state
pub struct HasFixed;    // After adding fixed-size data
pub struct HasVariable; // After adding variable-size data
pub struct Complete;    // Ready to build
```

#### Features
- **Compile-time validation**: Invalid field ordering won't compile
- **Size hints**: Specify sizes for variable-length fields
- **Padding support**: Automatically pads fields to specified sizes
- **Endianness support**: Methods for both big and little endian

#### Usage Example
```rust
use bebytes::builder::BytesBuilder;

// This compiles - correct order
let packet = BytesBuilder::new()
    .u8(0x01)              // Fixed: version
    .u16_be(0x0042)        // Fixed: flags (big-endian)
    .u32_le(0x12345678)    // Fixed: id (little-endian)
    .with_size(10)         // Size hint for next field
    .bytes(data)           // Variable: sized data
    .remaining_bytes(tail) // Variable: remaining data
    .build_variable();

// This won't compile - incorrect order
// let wrong = BytesBuilder::new()
//     .remaining_bytes(data)  // Variable first
//     .u8(0x01)              // ERROR: Can't add fixed after variable!
//     .build();
```

#### State Transitions
```
Empty → HasFixed → HasVariable → Complete
      ↓
   with_size() → SizedBuilder → HasVariable
```

## Benefits of TTT Approach

### 1. Type Safety
- Invalid states are unrepresentable
- Errors caught at compile time
- Self-documenting code through types

### 2. Better Error Messages
- Specific error types with context
- Field names in error messages
- Expected vs actual values

### 3. Ergonomic API
- Builder pattern guides correct usage
- Type system prevents mistakes
- IDE autocomplete works better with concrete types

### 4. Maintainability
- Clear separation of concerns
- Easy to extend with new error types
- State machine encoded in type system

## Example: Complete Usage

```rust
use bebytes::{BeBytes, BeBytesError, builder::BytesBuilder};

#[derive(BeBytes)]
struct Message {
    header: u32,
    len: u8,
    #[FromField(len)]
    data: Vec<u8>,
}

// Build a message
let bytes = BytesBuilder::new()
    .u32_be(0xDEADBEEF)    // header
    .u8(5)                 // len
    .with_size(5)
    .bytes(vec![1,2,3,4,5])
    .build_variable();

// Parse with proper error handling
match Message::try_from_be_bytes(&bytes) {
    Ok((msg, _)) => println!("Message: {:?}", msg),
    Err(BeBytesError::InvalidUtf8 { field }) => {
        eprintln!("UTF-8 error in {}", field);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Migration Guide

### Error Handling
No breaking changes - existing code continues to work. To leverage improved errors:

```rust
// Before
let result = MyStruct::try_from_be_bytes(&data)?;

// After - with specific error handling
let result = match MyStruct::try_from_be_bytes(&data) {
    Ok(r) => r,
    Err(BeBytesError::InsufficientData { expected, actual }) => {
        // Handle specific case
        return Err(MyError::NeedMoreData(expected - actual));
    }
    Err(e) => return Err(e.into()),
};
```

### Builder Pattern
The builder is a new, optional API. Use it when:
- Constructing test data
- Building packets programmatically
- Need compile-time field ordering validation

## Future Enhancements

Potential future improvements following TTT principles:

1. **Protocol-specific types**: `NullTerminatedString`, `PascalString`, etc.
2. **Validation traits**: Custom validation logic per type
3. **Transform pipelines**: Composable transformations (compression, encryption)
4. **Debug builders**: Builders that track field names for debugging

## Conclusion

The TTT pattern improvements make BeBytes:
- **Safer**: Type system prevents errors
- **Clearer**: Better error messages and API
- **More maintainable**: Separation of concerns through types

These changes maintain backward compatibility while providing opt-in improvements for users who want stronger type safety and better error handling.