# Test Organization

This directory contains all tests for the BeBytes library, organized by functionality.

## Structure

```
tests/
├── core.rs                    # Core functionality (primitives, arrays, structs)
├── bitfields.rs               # Bit field tests
├── enums.rs                   # Enum tests (basic, auto-sized, flags)
├── vectors.rs                 # Vector handling tests
├── size_expressions.rs        # Size expression tests (NEW in 2.3.0)
├── protocol_examples.rs       # Protocol tests with size expressions (NEW in 2.3.0)
├── errors.rs                  # Error handling tests
├── no_std.rs                  # no_std compatibility tests
├── integration.rs             # Complex real-world scenarios
├── compile_fail.rs            # Compile-time failure tests
├── property_tests.rs          # Property-based testing
├── macro_expansion.rs         # Macro expansion tests
├── bit_arithmetic.rs          # Bit arithmetic tests
├── check_optimization.rs      # Optimization verification
├── derive_critical.rs         # Critical derive functionality
├── evil_tests.rs              # Evil test scenarios
├── functional_coverage.rs     # Functional test coverage
├── arithmetic_mutations.rs    # Mutation testing: arithmetic
├── attribute_edge_cases.rs    # Attribute parsing edge cases
├── bitwise_mutations.rs       # Mutation testing: bitwise ops
├── comparison_mutations.rs    # Mutation testing: comparisons
├── logical_mutations.rs       # Mutation testing: logical ops
├── return_value_mutations.rs  # Mutation testing: return values
└── compile_time/              # Individual compile failure test cases
    ├── *.rs                   # Test cases that should fail
    └── *.stderr               # Expected error messages
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
- Bit fields with explicit sizes using `#[bits(N)]`
- Flag enums with `#[bebytes(flags)]`
- Bitwise operations
- Non-contiguous discriminants

### Vector Handling (`vectors.rs`)
- Fixed-size vectors with `#[With(size(N))]`
- Dynamic vectors with `#[FromField(field)]`
- Nested field access (e.g., `#[FromField(header.count)]`)
- Vectors as last field (padding)
- Custom type vectors

### Size Expressions (`size_expressions.rs`) - NEW in 2.3.0
- Mathematical operations (+, -, *, /, %)
- Field references and complex expressions
- String fields with dynamic sizing
- Nested expressions with parentheses
- Zero-size expression handling
- Error conditions and validation

### Protocol Examples (`protocol_examples.rs`) - NEW in 2.3.0
- IPv4/IPv6 packet structures
- DNS message parsing with variable sections
- MQTT packet with remaining length
- TCP segments with variable options
- HTTP-like messages with content-length
- Complex field dependencies

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

### Property-Based Tests (`property_tests.rs`)
- Randomized testing with quickcheck
- Round-trip serialization/deserialization
- Edge case generation
- Invariant checking

### Macro Expansion Tests (`macro_expansion.rs`)
- Verifies the derive macro generates expected methods
- Tests trait implementation completeness
- Constructor generation

### Bit Arithmetic Tests (`bit_arithmetic.rs`)
- Bit manipulation correctness
- Byte alignment calculations
- Division ceiling behavior
- Large bit field handling

### Mutation Testing Files
These files target specific mutation patterns to improve test quality:

- **`arithmetic_mutations.rs`**: Tests arithmetic operators (/, *, +, -, %)
- **`attribute_edge_cases.rs`**: Tests attribute parsing edge cases
- **`bitwise_mutations.rs`**: Tests bitwise operators (&, |, ^, <<, >>)
- **`comparison_mutations.rs`**: Tests comparison operators (==, !=, <, >)
- **`logical_mutations.rs`**: Tests logical operators (&&, ||)
- **`return_value_mutations.rs`**: Tests that functions return meaningful values
- **`derive_critical.rs`**: Tests critical derive functionality
- **`functional_coverage.rs`**: Tests functional code coverage

### Other Test Files
- **`check_optimization.rs`**: Verifies optimization behavior
- **`evil_tests.rs`**: Evil test scenarios to stress the system

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test category
cargo test core              # Test core functionality
cargo test bitfields         # Test bit fields
cargo test enums             # Test enums
cargo test vectors           # Test vectors
cargo test size_expressions  # Test size expressions (NEW in 2.3.0)
cargo test protocol_examples # Test protocol examples (NEW in 2.3.0)
cargo test errors            # Test error handling
cargo test integration       # Test complex scenarios

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