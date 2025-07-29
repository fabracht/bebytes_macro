# Changelog

All notable changes to this project will be documented in this file.

## [2.2.0] - Unreleased

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
- Pluggable string interpreter architecture via `StringInterpreter` trait
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