# Changelog

All notable changes to this project will be documented in this file.

## [2.9.0] - 2025-08-17

### Added
- **Marker Attributes**: Support for delimiter-based field parsing
  - `#[UntilMarker(byte_or_char)]` - Reads bytes until a specific marker is encountered
  - `#[AfterMarker(byte_or_char)]` - Skips bytes until finding a marker, then reads remaining data
  - Character literal support for ASCII characters (`'\n'`, `'\0'`, `'\t'`, `'\r'`, etc.)
  - Byte value support (0xFF, 0x00, etc.) for non-ASCII markers
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

### Architecture Changes
- bytes crate is now a **native dependency** (no feature flags required)
- `BytesMut` replaces `Vec<u8>` for internal buffer management
- Full backward compatibility maintained - all existing `Vec<u8>` methods unchanged
- Enhanced `std` and `no_std` support via bytes crate's feature system

### Features
- Zero-copy sharing via reference-counted `Bytes`
- Automatic memory cleanup
- Compatible with tokio, hyper, tonic, and async libraries
- Uses same buffer management as networking libraries

## [2.5.0] - 2025-07-30

### Added
- **Ultra-High-Performance Raw Pointer Methods**: Revolutionary encoding optimization for maximum speed
  - `supports_raw_pointer_encoding()` - Check if struct is eligible for raw pointer optimization
  - `RAW_POINTER_SIZE` - Compile-time constant for struct size
  - `encode_be_to_raw_stack()` / `encode_le_to_raw_stack()` - Safe stack-allocated encoding methods
  - `encode_be_to_raw_mut()` / `encode_le_to_raw_mut()` - Unsafe direct buffer writing methods
  - **40-80x performance improvement** over standard `to_be_bytes()` for eligible structs
  - **Zero allocations** with stack-based methods using compile-time sized arrays
  - **Compile-time safety** - array sizes determined automatically by macro
  - Direct memory writes using `std::ptr::copy_nonoverlapping` for maximum efficiency

### Performance
- Small structs (4 bytes): **60x speedup**
- Medium structs (16 bytes): **44x speedup**  
- Large structs (72 bytes): **28x speedup**
- Max structs (256 bytes): **5x speedup**
- Comprehensive benchmarking suite with extensive performance validation

### Safety
- Stack methods are completely safe with compile-time array sizing
- No runtime size checks needed - compiler enforces correctness
- Direct buffer methods include capacity validation
- Methods only generated for eligible structs, preventing misuse

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