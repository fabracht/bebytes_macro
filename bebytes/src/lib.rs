#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
pub use alloc::borrow::ToOwned;

pub use bebytes_derive::BeBytes;

/// Error type for `BeBytes` operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}
