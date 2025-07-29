//! String interpretation traits for pluggable encoding support

use crate::BeBytesError;

/// Trait for interpreting byte sequences as strings
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

/// Default UTF-8 interpreter
pub struct Utf8;

impl StringInterpreter for Utf8 {
    fn from_bytes(bytes: &[u8]) -> Result<String, BeBytesError> {
        core::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| BeBytesError::InvalidDiscriminant {
                value: 0,
                type_name: "String (invalid UTF-8)",
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