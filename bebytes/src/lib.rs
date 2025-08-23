//! # `BeBytes` - High-Performance Binary Serialization
//!
//! `BeBytes` is a Rust library that provides procedural macros for generating ultra-fast
//! serialization and deserialization methods for network structs. It supports both
//! big-endian and little-endian byte orders and includes advanced features like bit fields,
//! marker-delimited fields, and WebAssembly compatibility.
//!
//! ## Key Features
//!
//! - **High Performance**: Zero-copy operations and minimal allocations
//! - **Bit Fields**: Pack multiple fields into bytes with `#[bits(N)]` attribute
//! - **Marker Attributes**: Handle variable-length sections with `#[UntilMarker]` and `#[AfterMarker]`
//! - **Size Control**: Dynamic field sizing with `#[FromField]` and `#[With(size())]`
//! - **WebAssembly Support**: Full `no_std` compatibility for WASM targets
//! - **Comprehensive Types**: Support for primitives, strings, arrays, vectors, enums, and nested structs
//!
//! ## Quick Start
//!
//! ```rust
//! use bebytes::BeBytes;
//!
//! #[derive(BeBytes, Debug, PartialEq)]
//! struct NetworkMessage {
//!     #[bits(4)]
//!     version: u8,
//!     #[bits(4)]
//!     msg_type: u8,
//!     sender_id: u32,
//!     content_len: u8,
//!     #[FromField(content_len)]
//!     content: Vec<u8>,
//! }
//!
//! let message = NetworkMessage {
//!     version: 1,
//!     msg_type: 2,
//!     sender_id: 0x12345678,
//!     content_len: 5,
//!     content: b"hello".to_vec(),
//! };
//!
//! // Serialize
//! let bytes = message.to_be_bytes();
//!
//! // Deserialize
//! let (parsed, bytes_consumed) = NetworkMessage::try_from_be_bytes(&bytes).unwrap();
//! assert_eq!(parsed, message);
//! ```
//!
//! ## Marker-Delimited Fields
//!
//! Handle protocols with variable-length sections separated by marker bytes:
//!
//! ```rust
//! use bebytes::BeBytes;
//!
//! #[derive(BeBytes, Debug, PartialEq)]
//! struct Protocol {
//!     header: u32,
//!     #[UntilMarker(0xFF)]
//!     data: Vec<u8>,  // Reads until 0xFF marker
//!     footer: u16,
//! }
//!
//! // For multiple delimited sections:
//! #[derive(BeBytes, Debug, PartialEq)]
//! struct MultiSection {
//!     section_count: u8,
//!     #[FromField(section_count)]
//!     #[UntilMarker(0xFF)]
//!     sections: Vec<Vec<u8>>,  // Multiple sections, each ending with 0xFF
//! }
//! ```
//!
//! ## Error Handling
//!
//! All parsing operations return `Result<(T, usize), BeBytesError>` where:
//! - `T` is the parsed struct
//! - `usize` is the number of bytes consumed
//! - `BeBytesError` provides detailed error information

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
pub use alloc::borrow::ToOwned;

// Re-export Vec for use in generated code
#[cfg(not(feature = "std"))]
pub use alloc::vec::Vec;
#[cfg(feature = "std")]
pub use std::vec::Vec;

pub mod buffer;
pub mod interpreter;

pub use bebytes_derive::BeBytes;
#[doc(hidden)]
pub use interpreter::{StringInterpreter, Utf8};

// Re-export buffer types for generated code (previously from bytes crate)
pub use buffer::{BufMut, Bytes, BytesMut};

/// Error type for `BeBytes` operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeBytesError {
    /// Buffer is empty when data was expected
    EmptyBuffer,
    /// Not enough data in buffer
    InsufficientData { expected: usize, actual: usize },
    /// Invalid enum discriminant value
    InvalidDiscriminant { value: u8, type_name: &'static str },
    /// Bit field value exceeds maximum allowed
    InvalidBitField {
        value: u128,
        max: u128,
        field: &'static str,
    },
    /// Invalid UTF-8 sequence in string field
    InvalidUtf8 { field: &'static str },
    /// Marker byte not found when expected
    MarkerNotFound { marker: u8, field: &'static str },
    /// Field value out of range
    ValueOutOfRange {
        field: &'static str,
        value: String,
        max: String,
    },
}

impl core::fmt::Display for BeBytesError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyBuffer => write!(f, "No bytes provided"),
            Self::InsufficientData { expected, actual } => {
                write!(f, "Not enough bytes: expected {expected}, got {actual}")
            }
            Self::InvalidDiscriminant { value, type_name } => {
                write!(f, "Invalid discriminant {value} for type {type_name}")
            }
            Self::InvalidBitField { value, max, field } => {
                write!(f, "Value {value} exceeds maximum {max} for field {field}")
            }
            Self::InvalidUtf8 { field } => {
                write!(f, "Invalid UTF-8 sequence in field '{field}'")
            }
            Self::MarkerNotFound { marker, field } => {
                write!(f, "Marker byte 0x{marker:02X} not found in field '{field}'")
            }
            Self::ValueOutOfRange { field, value, max } => {
                write!(f, "Value {value} exceeds maximum {max} for field '{field}'")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BeBytesError {}

// Note: core::error::Error is stable since Rust 1.81
// We could add this when we update MSRV:
// #[cfg(not(feature = "std"))]
// impl core::error::Error for BeBytesError {}

pub trait BeBytes {
    fn field_size() -> usize;

    // Big-endian methods
    #[cfg(feature = "std")]
    fn to_be_bytes(&self) -> std::vec::Vec<u8>;

    #[cfg(not(feature = "std"))]
    fn to_be_bytes(&self) -> alloc::vec::Vec<u8>;

    /// Try to parse a struct from big-endian bytes
    ///
    /// # Errors
    ///
    /// Returns `BeBytesError::EmptyBuffer` if the input slice is empty
    /// Returns `BeBytesError::InsufficientData` if there aren't enough bytes to parse all fields
    /// Returns `BeBytesError::InvalidDiscriminant` if an enum field has an invalid value
    /// Returns `BeBytesError::InvalidBitField` if a bit field value exceeds its maximum
    fn try_from_be_bytes(bytes: &'_ [u8]) -> core::result::Result<(Self, usize), BeBytesError>
    where
        Self: Sized;

    // Little-endian methods
    #[cfg(feature = "std")]
    fn to_le_bytes(&self) -> std::vec::Vec<u8>;

    #[cfg(not(feature = "std"))]
    fn to_le_bytes(&self) -> alloc::vec::Vec<u8>;

    /// Convert to big-endian bytes as a Bytes buffer
    ///
    /// Returns a Bytes buffer (now a simple Vec wrapper without reference counting)
    fn to_be_bytes_buf(&self) -> Bytes {
        // Default implementation for backward compatibility
        Bytes::from(self.to_be_bytes())
    }

    /// Convert to little-endian bytes as a Bytes buffer
    ///
    /// Returns a Bytes buffer (now a simple Vec wrapper without reference counting)
    fn to_le_bytes_buf(&self) -> Bytes {
        // Default implementation for backward compatibility
        Bytes::from(self.to_le_bytes())
    }

    /// Try to parse a struct from little-endian bytes
    ///
    /// # Errors
    ///
    /// Returns `BeBytesError::EmptyBuffer` if the input slice is empty
    /// Returns `BeBytesError::InsufficientData` if there aren't enough bytes to parse all fields
    /// Returns `BeBytesError::InvalidDiscriminant` if an enum field has an invalid value
    /// Returns `BeBytesError::InvalidBitField` if a bit field value exceeds its maximum
    fn try_from_le_bytes(bytes: &'_ [u8]) -> core::result::Result<(Self, usize), BeBytesError>
    where
        Self: Sized;

    /// Encode directly to a buffer in big-endian format
    ///
    /// Writes struct data directly to the provided buffer without intermediate
    /// allocations.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer doesn't have enough capacity
    fn encode_be_to<B: BufMut>(&self, buf: &mut B) -> core::result::Result<(), BeBytesError> {
        // Default implementation using to_be_bytes for backward compatibility
        let bytes = self.to_be_bytes();
        buf.put_slice(&bytes);
        Ok(())
    }

    /// Encode directly to a buffer in little-endian format
    ///
    /// Writes struct data directly to the provided buffer without intermediate
    /// allocations.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer doesn't have enough capacity
    fn encode_le_to<B: BufMut>(&self, buf: &mut B) -> core::result::Result<(), BeBytesError> {
        // Default implementation using to_le_bytes for backward compatibility
        let bytes = self.to_le_bytes();
        buf.put_slice(&bytes);
        Ok(())
    }
}
