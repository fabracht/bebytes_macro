# Changelog

All notable changes to this project will be documented in this file.

## [3.0.0] - 2025-12-21

### Breaking Changes

- **Option wire format changed**: Options now use a tagged format for unambiguous serialization
  - `None` → `[0x00, zeros...]` (1 tag byte + N zero-padding bytes)
  - `Some(value)` → `[0x01, value...]` (1 tag byte + value bytes)
  - This fixes the ambiguity where `Some(0)` was indistinguishable from `None`
  - `field_size()` now returns inner type size + 1 byte for the tag
  - **Wire format is incompatible with previous versions**

### Added

- **Option<[u8; N]> array support**: Byte arrays can now be wrapped in Option
- **Option support for f32, f64, bool, char**: All primitive types now work with Option
- **InvalidChar error variant**: Proper error type for invalid Unicode code points in char fields
- Comprehensive test suite for all Option types (32 tests)
- Little-endian round-trip tests for Options

### Fixed

- `Some(0)` vs `None` disambiguation issue
- Invalid char validation now uses `InvalidChar` error instead of `InvalidDiscriminant`

## [2.11.0] - 2025-12-09

### Added

- **f32/f64 floating-point support**: Full serialization support for IEEE 754 floats
  - Round-trip serialization with `to_be_bytes()`/`to_le_bytes()`
  - Proper handling of special values: NaN, infinity, negative infinity
  - Endianness support for both big-endian and little-endian
- **bool primitive support**: Single-byte boolean serialization with strict validation
  - Serializes as `0x00` (false) or `0x01` (true)
  - Parsing rejects any byte value other than 0 or 1 with `InvalidDiscriminant` error
- Comprehensive test coverage for new primitive types

### Changed

- Removed unused `SUPPORTED_PRIMITIVES` constant and `is_supported_primitive_type` function
- Refactored `create_primitive_writing` for better maintainability

### Notes

- f32, f64, and bool cannot be used with `#[bits(N)]` attribute (compile-time error)
- bool uses strict validation unlike C-style "non-zero is true" semantics

## [2.10.0] - 2025-08-24

### Added

- **Character literal support for marker attributes**: ASCII characters can now be used in `#[UntilMarker]` and `#[AfterMarker]`
  - Support for common escape sequences: `'\n'`, `'\0'`, `'\t'`, `'\r'`
  - Compile-time validation ensures only ASCII characters (value <= 127) are used
- **MarkerNotFound error**: Proper error reporting when delimiter bytes are missing
  - Returns specific error with marker value and field name
  - Only errors for non-terminal fields (last field can consume remaining bytes)
  - Vec<Vec<u8>> always errors on missing markers for proper segment delimiting

### Fixed

- Pipeline failure in no_std builds due to missing String import in error types
- Improved error messages for missing marker delimiters

## [2.9.0] - 2025-08-17

### Added

- **Marker Attributes**: Support for delimiter-based field parsing
  - `#[UntilMarker(byte_or_char)]` - Reads bytes until a specific marker is encountered
  - `#[AfterMarker(byte_or_char)]` - Skips bytes until finding a marker, then reads remaining data
  - Byte value support (0xFF, 0x00, etc.)
  - Comprehensive support for protocols with sentinel-delimited sections
- **Vec<Vec<u8>> Support**: Multiple delimited sections handling
  - Works with `#[UntilMarker(byte_or_char)]` for nested byte sequences
  - Requires size control via `#[With(size(N))]` or `#[FromField(field_name)]`
  - Each inner Vec is terminated by the specified marker byte
  - Missing markers result in empty segments
- **Enhanced Error Types**: More specific error variants
  - `InvalidUtf8` for string encoding errors
  - `MarkerNotFound` for missing delimiter bytes in non-terminal fields
- Documentation improvements for marker attributes (MARKER_ATTRIBUTES.md)

### Fixed

- Added required-features for wasm_test example
- Applied cargo fmt and resolved clippy warnings

## [2.8.0] - 2025-08-11

### Changed

- **Removed bytes dependency**: Replaced with internal buffer module
  - Implemented internal `BytesMut` and `Bytes` types wrapping `Vec<u8>`
  - Created `BufMut` trait with all necessary buffer operations
  - Eliminated external dependency while maintaining full API compatibility
  - Reduced compile time and simplified dependency tree

### Performance

- Performance remains equivalent to previous bytes-based implementation
- Removed misleading "2x performance" claims (difference was just one memcpy)
- Actual benchmarks show similar performance for all operations

### Migration

- **No code changes required** - All APIs remain unchanged
- `to_be_bytes_buf()` and `to_le_bytes_buf()` continue to work
- `encode_be_to()` and `encode_le_to()` maintain same signatures
- Users who need `bytes::Bytes` can convert: `bytes::Bytes::from(vec)`

## [2.6.0] - 2025-07-30

### Added

- **bytes Crate Integration**: Native dependency for buffer management
  - `to_be_bytes_buf()` / `to_le_bytes_buf()` - `Bytes` buffer methods
  - `encode_be_to()` / `encode_le_to()` - Direct `BufMut` writing methods
  - `BytesMut` replaces `Vec<u8>` for internal buffer management
  - Integration with networking and async ecosystems

### Performance Improvements

- Slightly faster with `to_be_bytes_buf()` vs `to_be_bytes()` (avoids final memcpy)
- Direct `BufMut` writing provides cleaner API for buffer reuse
- `BytesMut::freeze()` → `Bytes` moves ownership without copying
- Primitive serialization using `BufMut::put_u8()`, `put_u16()`, etc.
- Pre-allocated buffers reduce allocations in batch operations

### Note

This version introduced `bytes` crate integration which was later replaced by internal buffer management in v2.8.0. See v2.8.0 changelog for current implementation details.

## [2.5.0] - 2025-07-30

### Added

- **Raw Pointer Methods**: Stack-allocated encoding for eligible structs
  - `supports_raw_pointer_encoding()` - Check if struct is eligible
  - `RAW_POINTER_SIZE` - Compile-time constant for struct size
  - `encode_be_to_raw_stack()` / `encode_le_to_raw_stack()` - Safe stack-allocated encoding
  - `encode_be_to_raw_mut()` / `encode_le_to_raw_mut()` - Unsafe direct buffer writing
  - Stack-based methods avoid heap allocation
  - Array sizes determined at compile time

### Safety

- Stack methods are safe with compile-time array sizing
- Direct buffer methods include capacity validation
- Methods only generated for eligible structs

### Eligibility

Raw pointer methods are available for structs that:

- Have no bit fields
- Are 256 bytes or smaller
- Contain only primitive types (u8-u128, i8-i128, char) and fixed-size u8 arrays

## [2.4.0] - 2025-07-30

### Added

- **Direct Buffer Writing**: New performance-oriented API for zero-allocation encoding
  - `encode_be_to()` and `encode_le_to()` methods for writing directly to `BufMut`
  - Feature-gated behind `bytes` feature flag
  - Eliminates intermediate Vec allocation in encoding path
  - Comprehensive test suite for direct buffer writing
  - Default implementation maintains backward compatibility

### Changed

- Added `#[inline]` annotations to all generated trait methods for better optimization
- Updated documentation with performance optimization section

## [2.3.0] - 2025-07-29

### Added

- **Size Expressions**: Dynamic field sizing using mathematical expressions
  - `#[With(size(expression))]` syntax for Vec<u8> and String fields
  - Support for mathematical operations: `+`, `-`, `*`, `/`, `%` with parentheses
  - Field references for dynamic sizing based on other struct fields
  - Complex expressions like `#[With(size((width * height) + padding))]`
  - Runtime expression evaluation with compile-time syntax validation
- Protocol examples demonstrating size expressions (IPv4, DNS, MQTT, TCP, HTTP)
- Comprehensive size expression documentation (SIZE_EXPRESSIONS.md)
- Size expression demonstration in macro_test.rs

### Changed

- Enhanced attribute parsing to support mathematical expressions
- Extended code generation for runtime size calculation
- Updated all documentation and examples to reflect new capabilities

## [2.2.0] - 2025-07-28

### Added

- Comprehensive string support with standard Rust `String` types
  - Fixed-size strings with `#[With(size(N))]`
  - Variable-size strings with `#[FromField(field_name)]`
  - Unbounded strings as the last field
- UTF-8 validation for all string deserialization
- String support documentation and examples
- Property-based testing with proptest
  - Round-trip serialization tests
  - Bit field bounds validation
  - Endianness consistency checks
  - String encoding/decoding verification
- Internal `StringInterpreter` trait for UTF-8 string handling (not user-facing)
- Separation of byte extraction from string interpretation

### Changed

- Simplified string API - removed `FixedString`, `VarString`, and `CString` types
- Reduced lib.rs from ~920 lines to 103 lines
- Minimum Rust version set to 1.73.0 for `div_ceil` support

### Fixed

- Replaced manual div_ceil implementation with standard library `.div_ceil(8)`
- Refactored functions exceeding 100 lines to comply with clippy pedantic warnings
- Fixed 8-argument function warning by using context struct pattern

## [2.1.2] - 2025-07-27

### Fixed

- Remove cfg attribute warnings from generated code in user crates

## [2.1.1] - 2025-07-27

### Fixed

- Vec path resolution in no_std environments for flag enums
- Re-export Vec from bebytes crate for both std and no_std

## [2.1.0] - 2025-07-27

### Added

- Flag decomposition functionality for flag enums
- `decompose()` method to convert u8 values into individual flag variants
- `iter_flags()` method for efficient iteration over set flags

### Fixed

- no_std compatibility for binary targets

## [2.0.0] - 2025-07-27

### Changed

- **BREAKING**: Removed auto-sized enum functionality
- Improved error types and handling
- Enhanced no_std compatibility

### Added

- Comprehensive test suite with mutation testing
- Better documentation and examples

## [1.5.0] - 2025-07-19

### Added

- Nested field access support for `#[FromField]` attribute
- Support for dot notation in field paths (e.g., `header.count`)

## [1.4.0] - 2025-07-18

### Added

- Performance optimizations from bebytes_derive 1.4.0
- 39% improvement in cross-byte bit operations
- Reduced memory allocations in vector operations

## [1.3.0] - 2025-07-18

### Added

- Enum discriminant validation
- Better error messages for invalid enum values

## [1.2.0] - 2025-07-17

### Added

- Auto-sized enum functionality
- Automatic bit width calculation for enums

## [1.1.0] - 2025-07-16

### Fixed

- Side effects removed from macro implementation
- Improved functional programming approach

## [1.0.0] - 2025-07-15

### Added

- Stable API release
- Full documentation
- Comprehensive test suite

## [0.7.1] - 2025-07-14

### Fixed

- Clippy warnings
- Documentation improvements

## [0.7.0] - 2025-07-14

### Added

- Support for vectors of custom types
- Better FromField attribute error messages

## [0.6.1] - 2025-07-14

### Fixed

- Minor documentation updates

## [0.6.0] - 2025-07-14

### Added

- Little-endian support
- `to_le_bytes()` and `try_from_le_bytes()` methods
- Complete endianness support for all types

## [0.5.0] - 2025-07-14

### Changed

- Major refactoring of codebase structure
- Improved test organization

### Fixed

- std feature flag handling

## [0.4.6] - 2025-07-14

### Fixed

- Updated bebytes_derive dependency

## [0.4.5] - 2025-07-14

### Fixed

- std feature compatibility

## [0.4.4] - 2025-07-14

### Changed

- Internal improvements

## [0.4.3] - 2025-07-14

### Changed

- Documentation updates

## [0.4.2] - 2025-07-13

### Fixed

- Minor bug fixes

## [0.4.1] - 2025-07-13

### Fixed

- Dependency updates

## [0.4.0] - 2025-07-13

### Added

- Vector support for custom types
- Improved error messages

### Fixed

- Formatting issues

## [0.3.1] - 2025-07-12

### Fixed

- Bug fixes in vector handling

## [0.3.0] - 2025-07-12

### Added

- Vector field support with size hints
- FromField attribute

## [0.2.1] - 2025-07-11

### Fixed

- Array serialization bug

## [0.2.0] - 2025-07-11

### Added

- Array support
- Better documentation

## [0.1.5] - 2025-07-10

### Fixed

- README updates
- Minor bug fixes

## [0.1.4] - 2025-07-10

### Fixed

- Expected size calculation bug

## [0.1.3] - 2025-07-10

### Added

- README documentation for derive crate

## [0.1.2] - 2025-07-10

### Fixed

- Repository URL in Cargo.toml

## [0.1.1] - 2025-07-10

### Fixed

- Workspace configuration
- Dependency updates

## [0.1.0] - 2025-07-10

### Added

- Initial release
- Basic BeBytes trait
- Primitive type support
- Bit field support
- Big-endian and little-endian serialization
