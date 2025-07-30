//! Internal string interpretation traits
//!
//! This module is for internal use by the `BeBytes` derive macro.
//! The traits and types are exposed publicly but are not intended for external use.
//! The derive macro is currently hardcoded to use UTF-8 encoding.

use crate::BeBytesError;

#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, string::String};

/// Internal trait for interpreting byte sequences as strings
///
/// This trait is exposed publicly but is intended for internal use only.
/// The derive macro currently only supports UTF-8 encoding.
#[doc(hidden)]
pub trait StringInterpreter {
    /// Convert bytes to a String
    ///
    /// # Errors
    ///
    /// Returns `BeBytesError::InvalidDiscriminant` if the bytes cannot be interpreted as a valid string
    fn from_bytes(bytes: &[u8]) -> Result<String, BeBytesError>;

    /// Convert a string to bytes
    fn to_bytes(s: &str) -> &[u8];
}

/// Internal UTF-8 interpreter implementation
///
/// This type is exposed publicly but is intended for internal use only.
#[doc(hidden)]
pub struct Utf8;

impl StringInterpreter for Utf8 {
    fn from_bytes(bytes: &[u8]) -> Result<String, BeBytesError> {
        core::str::from_utf8(bytes).map(str::to_owned).map_err(|_| {
            BeBytesError::InvalidDiscriminant {
                value: 0,
                type_name: "String (invalid UTF-8)",
            }
        })
    }

    fn to_bytes(s: &str) -> &[u8] {
        s.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_valid() {
        let bytes = b"Hello, world!";
        let result = Utf8::from_bytes(bytes);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, world!");
    }

    #[test]
    fn test_utf8_invalid() {
        let bytes = &[0xFF, 0xFE, 0xFD]; // Invalid UTF-8
        let result = Utf8::from_bytes(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_utf8_to_bytes() {
        let s = "Hello, world!";
        let bytes = Utf8::to_bytes(s);
        assert_eq!(bytes, b"Hello, world!");
    }
}
