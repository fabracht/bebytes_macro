# BeBytes Architecture

## Overview

BeBytes is a Rust procedural macro library for binary serialization/deserialization with bit-level precision. The library consists of two main crates:

1. **bebytes_derive** - The procedural macro implementation
2. **bebytes** - The trait definition and re-export

## Core Design Principles

### 1. Zero-Copy Where Possible
- Primitive types are read directly from byte slices
- Bit manipulation is done in-place without intermediate allocations
- Only allocates when necessary (strings, vectors)

### 2. Compile-Time Validation
- Bit field completeness is validated at compile time
- Type safety is enforced through the type system
- Clear error messages guide users to fix issues

### 3. Functional Programming Approach
- Pure functions for code generation
- Immutable data structures in macro processing
- Separation of parsing and code generation phases

## String Implementation

### Design Philosophy

The string implementation in BeBytes v2.3.0+ uses Rust's standard `String` type with attributes for size control.

### Implementation Details

#### String Field Types

Strings are identified in `determine_field_type` and categorized as:

```rust
enum FieldType {
    String(Option<usize>, Option<Vec<syn::Ident>>), // size, field_path
    // ... other variants
}
```

The two parameters represent:
- `Option<usize>`: Fixed size from `#[With(size(N))]`
- `Option<Vec<syn::Ident>>`: Field path from `#[FromField(field.name)]`

#### String Processing Functions

1. **Fixed-Size Strings** (`generate_fixed_size_string`)
   - Validates exact byte length during serialization
   - Panics if string doesn't match specified size
   - User responsible for padding

2. **Variable-Size Strings** (`generate_field_size_string`)
   - Reads size from another field at runtime
   - Supports nested field access (e.g., `header.name_len`)
   - No compile-time size validation

3. **Unbounded Strings** (`generate_unbounded_string`)
   - Only allowed as the last field
   - Consumes all remaining bytes
   - Common pattern for log messages or trailing data

#### UTF-8 Validation

All string deserialization includes UTF-8 validation:

```rust
let #field_name = match core::str::from_utf8(string_bytes) {
    Ok(s) => s.to_owned(),
    Err(_) => return Err(::bebytes::BeBytesError::InvalidDiscriminant {
        value: 0,
        type_name: "String (invalid UTF-8)",
    }),
};
```

### Memory Management

- Fixed-size strings pre-allocate exact capacity
- Variable-size strings allocate based on field value
- UTF-8 validation happens before allocation
- `to_owned()` creates owned String from validated &str

## Bit Field Architecture

### Bit Position Tracking

The macro maintains a `current_bit_position` throughout field processing to:
- Calculate byte boundaries
- Validate bit field completeness
- Generate correct bit masks

### Multi-Byte Bit Fields

For fields spanning multiple bytes:
1. Calculate byte boundaries using `.div_ceil(8)`
2. Generate aligned/unaligned parsing code
3. Handle endianness conversions
4. Apply bit masks for extraction

### Context Struct Pattern

Complex functions use context structs to avoid excessive parameters:

```rust
struct MultiByteBitFieldCtx<'a> {
    field_name: &'a syn::Ident,
    field_type: &'a syn::Type,
    size: usize,
    // ... other fields
}
```

## Code Generation Pipeline

### 1. Parsing Phase
- Extract attributes (`#[bits]`, `#[With]`, `#[FromField]`)
- Determine field types
- Validate constraints

### 2. Processing Phase
- Generate parsing code for each field
- Generate writing code for each field  
- Calculate bit sums for size computation

### 3. Assembly Phase
- Combine generated code fragments
- Add error handling
- Generate trait implementation

## Error Handling

BeBytes uses a custom error type with variants for:
- `InsufficientData`: Not enough bytes to parse
- `InvalidDiscriminant`: Invalid enum value or invalid UTF-8

Error information includes:
- Expected vs actual sizes
- Type information for debugging
- Proper error propagation with `?`

## Performance Considerations

### Optimizations
- Bit operations use native integer types
- Byte boundary alignment for direct memory access  
- Capacity pre-allocation for vectors and strings
- Minimal allocations during parsing

### Trade-offs
- String UTF-8 validation adds overhead but ensures safety
- Bit field flexibility vs raw byte performance
- Compile-time validation vs runtime flexibility

## String Interpretation (Internal)

### Design
BeBytes v2.3.0+ uses an internal `StringInterpreter` trait for string handling. This trait is currently for internal use only and always uses UTF-8 encoding.

### StringInterpreter Trait (Internal Use)
```rust
// Internal trait - not intended for external use
pub trait StringInterpreter {
    fn from_bytes(bytes: &[u8]) -> Result<String, BeBytesError>;
    fn to_bytes(s: &str) -> &[u8];
}
```

### Implementation
The `Utf8` struct provides UTF-8 interpretation. The derive macro is hardcoded to use this implementation.

### Generated Code
String parsing now generates:
```rust
let string_bytes = &bytes[byte_index..end_index];
let field_name = <::bebytes::Utf8 as ::bebytes::StringInterpreter>::from_bytes(string_bytes)?;
```

### Extension Points
Users can implement custom interpreters for:
- Alternative encodings (UTF-16, ASCII, etc.)
- Base64 encoded strings
- Custom domain-specific formats
- Integration with other serialization libraries

## Future Considerations

### Potential Improvements
- Attribute support for specifying interpreters
- Collection interpreters for Vec alternatives
- Character encoding options
- Integration with serde

### Backward Compatibility
- Default UTF-8 behavior unchanged
- All existing code continues to work
- New features are additive only