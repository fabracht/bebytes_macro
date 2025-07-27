# Test Organization

This directory contains all tests for the BeBytes library, organized by functionality.

## Structure

```
tests/
├── core.rs          # Core functionality (primitives, arrays, structs)
├── bitfields.rs     # Bit field tests
├── enums.rs         # Enum tests (basic, auto-sized, flags)
├── vectors.rs       # Vector handling tests
├── errors.rs        # Error handling tests
├── no_std.rs        # no_std compatibility tests
├── integration.rs   # Complex real-world scenarios
├── compile_fail.rs  # Compile-time failure tests
└── compile_time/    # Individual compile failure test cases
    ├── *.rs         # Test cases that should fail
    └── *.stderr     # Expected error messages
```

## Test Categories

### Core Functionality (`core.rs`)
- Primitive type serialization (u8-u128, i8-i128)
- Array handling
- Basic struct serialization
- Nested structs
- Endianness consistency

### Bit Fields (`bitfields.rs`)
- Single-byte bit fields
- Boundary-crossing bit fields
- Multi-byte bit fields
- Edge cases (single bits, max values)
- Overflow protection

### Enums (`enums.rs`)
- Basic enum serialization
- Auto-sized enums with `#[bits()]`
- Flag enums with `#[bebytes(flags)]`
- Bitwise operations
- Non-contiguous discriminants

### Vector Handling (`vectors.rs`)
- Fixed-size vectors with `#[With(size(N))]`
- Dynamic vectors with `#[FromField(field)]`
- Nested field access (e.g., `#[FromField(header.count)]`)
- Vectors as last field (padding)
- Custom type vectors

### Error Handling (`errors.rs`)
- Error display formatting
- All error variants (EmptyBuffer, InsufficientData, etc.)
- Custom Result type alias compatibility
- Error propagation in nested structures

### No-std Support (`no_std.rs`)
- Tests that run without standard library
- Verifies all features work in embedded contexts

### Integration Tests (`integration.rs`)
- Complete packet protocols
- TLV (Type-Length-Value) structures
- Complex nested structures
- Real-world protocol implementations
- Performance scenarios with large batches

### Compile-Time Tests (`compile_fail.rs`)
- Verifies invalid code is rejected at compile time
- Uses trybuild framework
- Tests for unsupported types, incomplete bytes, etc.

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test category
cargo test core          # Test core functionality
cargo test bitfields     # Test bit fields
cargo test enums         # Test enums
cargo test vectors       # Test vectors
cargo test errors        # Test error handling
cargo test integration   # Test complex scenarios

# Run no_std tests
cargo test --no-default-features no_std

# Run compile-time tests
cargo test compile_fail

# Run with verbose output
cargo test -- --nocapture

# Run a specific test function
cargo test test_simple_struct

# Run tests from a specific file
cargo test --test core

# Run tests matching a pattern
cargo test enum   # Runs all enum-related tests
cargo test vector # Runs vector handling tests
```