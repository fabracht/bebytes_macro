# BeBytes Compile-Time Test Suite

This directory contains compile-time tests that verify the BeBytes derive macro properly rejects invalid code at compile time. These tests use the `trybuild` crate to ensure error messages are clear and helpful.

## Organization

Tests are organized into categories based on the type of error they validate:

### 📁 `attributes/`
Tests for attribute conflicts and invalid attribute combinations.

- `bits_and_size_conflict.rs` - Cannot use both `#[bits]` and `#[bebytes(size)]`
- `fromfield_and_with_conflict.rs` - Cannot use both `#[FromField]` and `#[With(size())]`
- `multiple_endian_attrs.rs` - Cannot specify both big and little endian

### 📁 `bit_fields/`
Tests for bit field validation and errors.

- `zero_bits.rs` - Bit fields must have at least 1 bit
- `exceeds_type_size.rs` - Bit count cannot exceed type capacity (e.g., 9 bits on u8)
- `bits_on_non_numeric.rs` - `#[bits]` only works on numeric types
- `multiple_bits_attributes.rs` - Cannot have multiple `#[bits]` on same field
- `incomplete_byte.rs` - Bit fields must complete full bytes

### 📁 `enums/`
Tests for enum-specific constraints and errors.

- `missing_repr_u8.rs` - Enums must have `#[repr(u8)]`
- `duplicate_discriminants.rs` - No duplicate discriminant values
- `data_variants.rs` - Only unit variants supported
- `enum_discriminant_too_large.rs` - Discriminants must fit in u8 (0-255)
- `invalid_flag_enum.rs` - Flag enum values must be powers of 2
- `flag_enum_too_large.rs` - Flag enum values must fit in u8
- `zero_value_flag_enum.rs` - Flag enums can have zero value (passes)

### 📁 `markers/`
Tests for marker attribute validation.

- `non_ascii_char.rs` - Character markers must be ASCII (value <= 127)

### 📁 `size_expressions/`
Tests for size expression parsing and validation.

- `nonexistent_field.rs` - Referenced fields must exist
- `circular_dependency.rs` - Detects circular size dependencies
- `division_by_zero.rs` - Catches division by zero at compile time
- `invalid_operator.rs` - Only arithmetic operators allowed

### 📁 `types/`
Tests for unsupported types and type constraints.

- `unsupported_structure.rs` - Only named structs supported
- `unsupported_f64.rs` - Floating point types not supported
- `unsupported_isize.rs` - Pointer-sized integers not supported

### 📁 `vectors/`
Tests for vector field rules and constraints.

- `fromfield_nonexistent.rs` - `#[FromField]` must reference existing field
- `fromfield_non_numeric.rs` - Size fields must be numeric
- `multiple_vecs_no_size.rs` - Can't have multiple unsized vectors
- `vec_not_last_no_size.rs` - Unsized vector must be last field
- `safe_nested_vector.rs` - Properly sized vectors (passes)

## Test Files That Pass

Some test files in this directory are expected to compile successfully:
- `arrayed.rs` - Array handling
- `custom_result_alias.rs` - Custom Result type aliases
- `nested_struct.rs` - Nested struct support
- `option.rs` - Option type support
- `test_chars.rs` - Character type support
- `test_u8s.rs`, `test_u16s.rs`, `test_u32s.rs` - Numeric types
- `unnamed_fields.rs` - Unnamed fields with proper attributes
- `zero_value_flag_enum.rs` - Zero is valid in flag enums

## Running Tests

```bash
# Run all compile-time tests
cargo test compile_fail

# Run with verbose output to see which tests are being executed
cargo test compile_fail -- --nocapture

# Update expected error messages (when intentionally changing error output)
TRYBUILD=overwrite cargo test compile_fail
```

## Adding New Tests

When adding a new compile-time test:

1. **Choose the appropriate category** based on what the test validates
2. **Create a descriptive filename** that explains the error being tested
3. **Add a header comment** explaining what the test validates and why
4. **Write minimal code** that triggers only the specific error
5. **Run the test** to capture the error message
6. **Create a `.stderr` file** with the expected error output
7. **Update `compile_fail.rs`** to include the new test

### Example Test Structure

```rust
// This test verifies that [specific condition] produces a compile-time error
// because [explanation of why this should be an error].

use bebytes::BeBytes;

#[derive(BeBytes)]
struct InvalidExample {
    // Code that should trigger the error
}

fn main() {}
```

## Error Message Guidelines

Good compile-time errors should:
- Clearly state what went wrong
- Point to the exact location of the error
- Suggest how to fix the problem when possible
- Use consistent terminology

## Maintenance

- Review error messages when updating the derive macro
- Ensure new features have corresponding negative tests
- Keep tests minimal and focused on one error each
- Update this README when adding new test categories