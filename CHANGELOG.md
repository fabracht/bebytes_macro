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

### Changed
- Simplified string API - removed `FixedString`, `VarString`, and `CString` types
- Reduced lib.rs from ~920 lines to 103 lines
- Minimum Rust version set to 1.73.0 for `div_ceil` support

### Fixed
- Replaced manual div_ceil implementation with standard library `.div_ceil(8)`
- Refactored functions exceeding 100 lines to comply with clippy pedantic warnings
- Fixed 8-argument function warning by using context struct pattern

### Technical Improvements
- Better code organization with extracted helper functions
- Improved maintainability through function decomposition
- Context struct pattern for complex parameter passing
- Pluggable string interpreter architecture via `StringInterpreter` trait
- Separation of byte extraction from string interpretation

## [2.1.2] - Previous Release

### Fixed
- Vec path resolution in no_std environments