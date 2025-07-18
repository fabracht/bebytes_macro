# Test Organization

This directory contains all tests for the BeBytes library.

## Structure

```
tests/
├── compile_time/                        # Tests that verify compile-time behavior
│   ├── *.rs                            # Test cases that should fail compilation
│   └── *.stderr                        # Expected error messages
├── test_arrays.rs                      # Array type serialization tests
├── test_auto_enum_bits.rs              # Auto-sized enum field tests
├── test_auto_enum_bits_comprehensive.rs # Comprehensive auto enum tests
├── test_bit_fields.rs                  # Bit field functionality tests
├── test_complex_scenarios.rs           # Integration tests for complex scenarios
├── test_enum_bits.rs                   # Enum bit packing tests
├── test_enum_flags.rs                  # Flag enum tests (bitwise operations)
├── test_nested_structs.rs              # Nested struct serialization tests
├── test_vector_handling.rs             # Vector handling tests
├── compile_time_tests.rs               # TryBuild runner for compile-time tests
└── runtime_tests.rs                    # Parameterized tests for edge cases
```

## Test Categories

### Compile-Time Tests (`compile_time/`)
- Tests that verify the macro correctly rejects invalid code
- Uses TryBuild framework to test compilation failures
- Each `.rs` file has a corresponding `.stderr` file with expected errors

### Feature-Specific Tests
- **Arrays** (`test_arrays.rs`): Tests for array serialization
- **Bit Fields** (`test_bit_fields.rs`): Tests for `#[bits(N)]` functionality
- **Enums** (`test_enum_*.rs`): Tests for enum serialization, auto-sizing, and flags
- **Nested Structs** (`test_nested_structs.rs`): Tests for complex nested structures
- **Vectors** (`test_vector_handling.rs`): Tests for `#[FromField]` and `#[With(size())]`

### Integration Tests
- **Complex Scenarios** (`test_complex_scenarios.rs`): Real-world packet structures, TLV fields

### Edge Cases
- **Runtime Tests** (`runtime_tests.rs`): Parameterized tests using test-case

## Running Tests

```bash
# Run all tests
cargo test

# Run only compile-time tests
cargo test --test compile_time_tests

# Run specific test file
cargo test --test test_bit_fields
cargo test --test test_enum_flags

# Run tests matching a pattern
cargo test enum   # Runs all enum-related tests
cargo test vector # Runs vector handling tests
```