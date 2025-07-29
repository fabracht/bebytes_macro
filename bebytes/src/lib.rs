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

/// A fixed-length string type that can be serialized with BeBytes.
///
/// This type wraps a byte array and provides UTF-8 string functionality
/// with automatic padding and truncation to fit the fixed size.
///
/// # Examples
///
/// ```
/// use bebytes::{BeBytes, FixedString};
///
/// #[derive(BeBytes, Debug, PartialEq)]
/// struct Message {
///     name: FixedString<16>,
///     status: FixedString<8>,
/// }
///
/// let msg = Message {
///     name: FixedString::from_str("Alice"),
///     status: FixedString::from_str("active"),
/// };
///
/// let bytes = msg.to_be_bytes();
/// let (decoded, _) = Message::try_from_be_bytes(&bytes).unwrap();
/// assert_eq!(decoded, msg);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FixedString<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> FixedString<N> {
    /// Create a new FixedString filled with null bytes
    pub const fn new() -> Self {
        Self { data: [0; N] }
    }

    /// Create a FixedString from a string slice
    ///
    /// If the string is longer than N bytes, it will be truncated.
    /// If shorter, it will be padded with null bytes.
    pub fn from_str(s: &str) -> Self {
        let mut data = [0u8; N];
        let bytes = s.as_bytes();
        let copy_len = core::cmp::min(bytes.len(), N);
        data[..copy_len].copy_from_slice(&bytes[..copy_len]);
        Self { data }
    }

    /// Create a FixedString from a String
    ///
    /// If the string is longer than N bytes, it will be truncated.
    /// If shorter, it will be padded with null bytes.
    #[cfg(feature = "std")]
    pub fn from_string(s: String) -> Self {
        Self::from_str(&s)
    }

    /// Create a FixedString from a String (no_std version)
    #[cfg(not(feature = "std"))]
    pub fn from_string(s: alloc::string::String) -> Self {
        Self::from_str(&s)
    }

    /// Get the underlying byte array
    pub const fn as_bytes(&self) -> &[u8; N] {
        &self.data
    }

    /// Get a mutable reference to the underlying byte array
    pub fn as_bytes_mut(&mut self) -> &mut [u8; N] {
        &mut self.data
    }

    /// Convert to a string slice, stopping at the first null byte
    ///
    /// Returns None if the data contains invalid UTF-8
    pub fn as_str(&self) -> Option<&str> {
        let null_pos = self.data.iter().position(|&b| b == 0).unwrap_or(N);
        core::str::from_utf8(&self.data[..null_pos]).ok()
    }

    /// Convert to a String, stopping at the first null byte
    ///
    /// Returns None if the data contains invalid UTF-8
    #[cfg(feature = "std")]
    pub fn to_string(&self) -> Option<std::string::String> {
        self.as_str().map(|s| s.to_owned())
    }

    /// Convert to a String, stopping at the first null byte (no_std version)
    ///
    /// Returns None if the data contains invalid UTF-8
    #[cfg(not(feature = "std"))]
    pub fn to_string(&self) -> Option<alloc::string::String> {
        self.as_str().map(|s| s.to_owned())
    }

    /// Get the length of the string (up to the first null byte)
    pub fn len(&self) -> usize {
        self.data.iter().position(|&b| b == 0).unwrap_or(N)
    }

    /// Check if the string is empty (starts with null byte)
    pub fn is_empty(&self) -> bool {
        self.data[0] == 0
    }

    /// Clear the string (fill with null bytes)
    pub fn clear(&mut self) {
        self.data.fill(0);
    }
}

impl<const N: usize> Default for FixedString<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> From<&str> for FixedString<N> {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

#[cfg(feature = "std")]
impl<const N: usize> From<std::string::String> for FixedString<N> {
    fn from(s: std::string::String) -> Self {
        Self::from_string(s)
    }
}

#[cfg(not(feature = "std"))]
impl<const N: usize> From<alloc::string::String> for FixedString<N> {
    fn from(s: alloc::string::String) -> Self {
        Self::from_string(s)
    }
}

impl<const N: usize> core::fmt::Display for FixedString<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.as_str() {
            Some(s) => write!(f, "{}", s),
            None => write!(f, "<invalid UTF-8>"),
        }
    }
}

// Implement BeBytes for FixedString by delegating to the underlying byte array
impl<const N: usize> BeBytes for FixedString<N> {
    fn field_size() -> usize {
        N
    }

    #[cfg(feature = "std")]
    fn to_be_bytes(&self) -> std::vec::Vec<u8> {
        self.data.to_vec()
    }

    #[cfg(not(feature = "std"))]
    fn to_be_bytes(&self) -> alloc::vec::Vec<u8> {
        self.data.to_vec()
    }

    fn try_from_be_bytes(bytes: &'_ [u8]) -> core::result::Result<(Self, usize), BeBytesError>
    where
        Self: Sized,
    {
        if bytes.len() < N {
            return Err(BeBytesError::InsufficientData {
                expected: N,
                actual: bytes.len(),
            });
        }

        let mut data = [0u8; N];
        data.copy_from_slice(&bytes[..N]);

        Ok((Self { data }, N))
    }

    #[cfg(feature = "std")]
    fn to_le_bytes(&self) -> std::vec::Vec<u8> {
        // For byte arrays, endianness doesn't matter
        self.data.to_vec()
    }

    #[cfg(not(feature = "std"))]
    fn to_le_bytes(&self) -> alloc::vec::Vec<u8> {
        // For byte arrays, endianness doesn't matter
        self.data.to_vec()
    }

    fn try_from_le_bytes(bytes: &'_ [u8]) -> core::result::Result<(Self, usize), BeBytesError>
    where
        Self: Sized,
    {
        // For byte arrays, endianness doesn't matter
        Self::try_from_be_bytes(bytes)
    }
}

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
