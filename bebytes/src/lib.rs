#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
pub use alloc::borrow::ToOwned;
#[cfg(not(feature = "std"))]
use alloc::vec;

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
/// use bebytes::{BeBytes, FixedString16, FixedString8};
///
/// #[derive(BeBytes, Debug, PartialEq)]
/// struct Message {
///     name: FixedString16,
///     status: FixedString8,
/// }
///
/// let msg = Message {
///     name: FixedString16::from_str("Alice"),
///     status: FixedString8::from_str("active"),
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

/// A variable-length string type with a length prefix that can be serialized with BeBytes.
///
/// This type stores a UTF-8 string with an automatic length prefix. The length is stored
/// as the specified integer type (u8, u16, or u32) followed by the UTF-8 bytes.
///
/// # Examples
///
/// ```
/// use bebytes::{BeBytes, VarString8};
///
/// #[derive(BeBytes, Debug, PartialEq)]
/// struct Message {
///     id: u32,
///     content: VarString8,  // Max 255 bytes
/// }
///
/// let msg = Message {
///     id: 42,
///     content: VarString8::from_str("Hello, world!"),
/// };
///
/// let bytes = msg.to_be_bytes();
/// let (decoded, _) = Message::try_from_be_bytes(&bytes).unwrap();
/// assert_eq!(decoded, msg);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VarString<T> {
    data: Vec<u8>,
    _phantom: core::marker::PhantomData<T>,
}

impl<T> VarString<T> {
    /// Create a new empty VarString
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Create a VarString from a string slice
    pub fn from_str(s: &str) -> Self {
        Self {
            data: s.as_bytes().to_vec(),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Create a VarString from a String
    #[cfg(feature = "std")]
    pub fn from_string(s: String) -> Self {
        Self {
            data: s.into_bytes(),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Create a VarString from a String (no_std version)
    #[cfg(not(feature = "std"))]
    pub fn from_string(s: alloc::string::String) -> Self {
        Self {
            data: s.into_bytes(),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Get the underlying byte data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get a mutable reference to the underlying byte data
    pub fn as_bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    /// Convert to a string slice
    ///
    /// Returns None if the data contains invalid UTF-8
    pub fn as_str(&self) -> Option<&str> {
        core::str::from_utf8(&self.data).ok()
    }

    /// Convert to a String
    ///
    /// Returns None if the data contains invalid UTF-8
    #[cfg(feature = "std")]
    pub fn to_string(&self) -> Option<std::string::String> {
        self.as_str().map(|s| s.to_owned())
    }

    /// Convert to a String (no_std version)
    ///
    /// Returns None if the data contains invalid UTF-8
    #[cfg(not(feature = "std"))]
    pub fn to_string(&self) -> Option<alloc::string::String> {
        self.as_str().map(|s| s.to_owned())
    }

    /// Get the length of the string in bytes
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear the string
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Push a string slice to the end
    pub fn push_str(&mut self, s: &str) {
        self.data.extend_from_slice(s.as_bytes());
    }

    /// Push a character to the end
    pub fn push(&mut self, ch: char) {
        let mut buf = [0; 4];
        let s = ch.encode_utf8(&mut buf);
        self.push_str(s);
    }
}

impl<T> Default for VarString<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<&str> for VarString<T> {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

#[cfg(feature = "std")]
impl<T> From<std::string::String> for VarString<T> {
    fn from(s: std::string::String) -> Self {
        Self::from_string(s)
    }
}

#[cfg(not(feature = "std"))]
impl<T> From<alloc::string::String> for VarString<T> {
    fn from(s: alloc::string::String) -> Self {
        Self::from_string(s)
    }
}

impl<T> core::fmt::Display for VarString<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.as_str() {
            Some(s) => write!(f, "{}", s),
            None => write!(f, "<invalid UTF-8>"),
        }
    }
}

/// Type aliases for common fixed-length string sizes
pub type FixedString8 = FixedString<8>;
pub type FixedString16 = FixedString<16>;
pub type FixedString32 = FixedString<32>;
pub type FixedString64 = FixedString<64>;

/// Type aliases for common variable-length string sizes
pub type VarString8 = VarString<u8>; // Max 255 bytes
pub type VarString16 = VarString<u16>; // Max 65535 bytes
pub type VarString32 = VarString<u32>; // Max 4GB bytes

/// A C-style null-terminated string type that can be serialized with BeBytes.
///
/// This type represents strings that are terminated by a null byte (0x00), commonly
/// used in C APIs and legacy systems. The string data is stored without a length prefix,
/// relying on the null terminator to determine the end.
///
/// # Examples
///
/// ```
/// use bebytes::{BeBytes, CString};
///
/// #[derive(BeBytes, Debug, PartialEq)]
/// struct CStyleMessage {
///     name: CString,
///     path: CString,
/// }
///
/// let msg = CStyleMessage {
///     name: CString::from_str("alice"),
///     path: CString::from_str("/home/alice"),
/// };
///
/// let bytes = msg.to_be_bytes();
/// let (decoded, _) = CStyleMessage::try_from_be_bytes(&bytes).unwrap();
/// assert_eq!(decoded, msg);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CString {
    data: Vec<u8>,
}

impl CString {
    /// Create a new empty CString
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Create a CString from a string slice
    ///
    /// Note: Null bytes in the input string will be treated as terminators
    pub fn from_str(s: &str) -> Self {
        let bytes = s.as_bytes();
        // Find the first null byte if any, and truncate there
        let end_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        Self {
            data: bytes[..end_pos].to_vec(),
        }
    }

    /// Create a CString from a String
    #[cfg(feature = "std")]
    pub fn from_string(s: String) -> Self {
        Self::from_str(&s)
    }

    /// Create a CString from a String (no_std version)
    #[cfg(not(feature = "std"))]
    pub fn from_string(s: alloc::string::String) -> Self {
        Self::from_str(&s)
    }

    /// Get the underlying byte data (without null terminator)
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get a mutable reference to the underlying byte data
    pub fn as_bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    /// Convert to a string slice
    ///
    /// Returns None if the data contains invalid UTF-8
    pub fn as_str(&self) -> Option<&str> {
        core::str::from_utf8(&self.data).ok()
    }

    /// Convert to a String
    ///
    /// Returns None if the data contains invalid UTF-8
    #[cfg(feature = "std")]
    pub fn to_string(&self) -> Option<std::string::String> {
        self.as_str().map(|s| s.to_owned())
    }

    /// Convert to a String (no_std version)
    ///
    /// Returns None if the data contains invalid UTF-8
    #[cfg(not(feature = "std"))]
    pub fn to_string(&self) -> Option<alloc::string::String> {
        self.as_str().map(|s| s.to_owned())
    }

    /// Get the length of the string in bytes
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear the string
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Push a string slice to the end
    pub fn push_str(&mut self, s: &str) {
        // Only add bytes up to the first null byte
        let bytes = s.as_bytes();
        let end_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        self.data.extend_from_slice(&bytes[..end_pos]);
    }

    /// Push a character to the end
    pub fn push(&mut self, ch: char) {
        if ch != '\0' {
            // Don't allow null characters
            let mut buf = [0; 4];
            let s = ch.encode_utf8(&mut buf);
            self.push_str(s);
        }
    }
}

impl Default for CString {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for CString {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

#[cfg(feature = "std")]
impl From<std::string::String> for CString {
    fn from(s: std::string::String) -> Self {
        Self::from_string(s)
    }
}

#[cfg(not(feature = "std"))]
impl From<alloc::string::String> for CString {
    fn from(s: alloc::string::String) -> Self {
        Self::from_string(s)
    }
}

impl core::fmt::Display for CString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.as_str() {
            Some(s) => write!(f, "{}", s),
            None => write!(f, "<invalid UTF-8>"),
        }
    }
}

// Implement BeBytes for CString
impl BeBytes for CString {
    fn field_size() -> usize {
        // Variable size - this is not accurate, but required by trait
        // CString size depends on content + 1 byte for null terminator
        0
    }

    #[cfg(feature = "std")]
    fn to_be_bytes(&self) -> std::vec::Vec<u8> {
        let mut result = self.data.clone();
        result.push(0); // Add null terminator
        result
    }

    #[cfg(not(feature = "std"))]
    fn to_be_bytes(&self) -> alloc::vec::Vec<u8> {
        let mut result = self.data.clone();
        result.push(0); // Add null terminator
        result
    }

    fn try_from_be_bytes(bytes: &'_ [u8]) -> core::result::Result<(Self, usize), BeBytesError>
    where
        Self: Sized,
    {
        // Find the null terminator
        let null_pos = bytes.iter().position(|&b| b == 0);

        match null_pos {
            Some(pos) => {
                let string_data = &bytes[..pos];

                // Validate UTF-8
                if core::str::from_utf8(string_data).is_err() {
                    return Err(BeBytesError::InvalidDiscriminant {
                        value: 0, // Not really a discriminant, but reusing error type
                        type_name: "CString (invalid UTF-8)",
                    });
                }

                Ok((
                    Self {
                        data: string_data.to_vec(),
                    },
                    pos + 1,
                )) // +1 to include the null terminator in bytes consumed
            }
            None => {
                // No null terminator found - this is an error for C strings
                Err(BeBytesError::InvalidDiscriminant {
                    value: 0,
                    type_name: "CString (missing null terminator)",
                })
            }
        }
    }

    #[cfg(feature = "std")]
    fn to_le_bytes(&self) -> std::vec::Vec<u8> {
        // For byte data, endianness doesn't matter
        self.to_be_bytes()
    }

    #[cfg(not(feature = "std"))]
    fn to_le_bytes(&self) -> alloc::vec::Vec<u8> {
        // For byte data, endianness doesn't matter
        self.to_be_bytes()
    }

    fn try_from_le_bytes(bytes: &'_ [u8]) -> core::result::Result<(Self, usize), BeBytesError>
    where
        Self: Sized,
    {
        // For byte data, endianness doesn't matter
        Self::try_from_be_bytes(bytes)
    }
}

// Helper trait to handle different length prefix types
trait LengthPrefix: Copy {
    fn from_usize(len: usize) -> Option<Self>;
    fn to_usize(self) -> usize;
    fn size() -> usize;
    fn from_be_bytes(bytes: &[u8]) -> Self;
    fn to_be_bytes(self) -> Vec<u8>;
    fn from_le_bytes(bytes: &[u8]) -> Self;
    fn to_le_bytes(self) -> Vec<u8>;
}

impl LengthPrefix for u8 {
    fn from_usize(len: usize) -> Option<Self> {
        if len <= u8::MAX as usize {
            Some(len as u8)
        } else {
            None
        }
    }

    fn to_usize(self) -> usize {
        self as usize
    }

    fn size() -> usize {
        1
    }

    fn from_be_bytes(bytes: &[u8]) -> Self {
        bytes[0]
    }

    fn to_be_bytes(self) -> Vec<u8> {
        vec![self]
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        bytes[0]
    }

    fn to_le_bytes(self) -> Vec<u8> {
        vec![self]
    }
}

impl LengthPrefix for u16 {
    fn from_usize(len: usize) -> Option<Self> {
        if len <= u16::MAX as usize {
            Some(len as u16)
        } else {
            None
        }
    }

    fn to_usize(self) -> usize {
        self as usize
    }

    fn size() -> usize {
        2
    }

    fn from_be_bytes(bytes: &[u8]) -> Self {
        u16::from_be_bytes([bytes[0], bytes[1]])
    }

    fn to_be_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        u16::from_le_bytes([bytes[0], bytes[1]])
    }

    fn to_le_bytes(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl LengthPrefix for u32 {
    fn from_usize(len: usize) -> Option<Self> {
        if len <= u32::MAX as usize {
            Some(len as u32)
        } else {
            None
        }
    }

    fn to_usize(self) -> usize {
        self as usize
    }

    fn size() -> usize {
        4
    }

    fn from_be_bytes(bytes: &[u8]) -> Self {
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn to_be_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn to_le_bytes(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

// Implement BeBytes for VarString with different length prefix types
impl<T: LengthPrefix> BeBytes for VarString<T> {
    fn field_size() -> usize {
        // Variable size - this is not accurate, but required by trait
        // In practice, the size depends on the content
        0
    }

    #[cfg(feature = "std")]
    fn to_be_bytes(&self) -> std::vec::Vec<u8> {
        let len_prefix =
            T::from_usize(self.data.len()).expect("String too long for length prefix type");
        let mut result = len_prefix.to_be_bytes();
        result.extend_from_slice(&self.data);
        result
    }

    #[cfg(not(feature = "std"))]
    fn to_be_bytes(&self) -> alloc::vec::Vec<u8> {
        let len_prefix =
            T::from_usize(self.data.len()).expect("String too long for length prefix type");
        let mut result = len_prefix.to_be_bytes();
        result.extend_from_slice(&self.data);
        result
    }

    fn try_from_be_bytes(bytes: &'_ [u8]) -> core::result::Result<(Self, usize), BeBytesError>
    where
        Self: Sized,
    {
        let prefix_size = T::size();

        if bytes.len() < prefix_size {
            return Err(BeBytesError::InsufficientData {
                expected: prefix_size,
                actual: bytes.len(),
            });
        }

        let length = T::from_be_bytes(&bytes[..prefix_size]).to_usize();
        let total_size = prefix_size + length;

        if bytes.len() < total_size {
            return Err(BeBytesError::InsufficientData {
                expected: total_size,
                actual: bytes.len(),
            });
        }

        let string_data = &bytes[prefix_size..total_size];

        // Validate UTF-8
        if core::str::from_utf8(string_data).is_err() {
            return Err(BeBytesError::InvalidDiscriminant {
                value: 0, // Not really a discriminant, but reusing error type
                type_name: "VarString (invalid UTF-8)",
            });
        }

        Ok((
            Self {
                data: string_data.to_vec(),
                _phantom: core::marker::PhantomData,
            },
            total_size,
        ))
    }

    #[cfg(feature = "std")]
    fn to_le_bytes(&self) -> std::vec::Vec<u8> {
        let len_prefix =
            T::from_usize(self.data.len()).expect("String too long for length prefix type");
        let mut result = len_prefix.to_le_bytes();
        result.extend_from_slice(&self.data);
        result
    }

    #[cfg(not(feature = "std"))]
    fn to_le_bytes(&self) -> alloc::vec::Vec<u8> {
        let len_prefix =
            T::from_usize(self.data.len()).expect("String too long for length prefix type");
        let mut result = len_prefix.to_le_bytes();
        result.extend_from_slice(&self.data);
        result
    }

    fn try_from_le_bytes(bytes: &'_ [u8]) -> core::result::Result<(Self, usize), BeBytesError>
    where
        Self: Sized,
    {
        let prefix_size = T::size();

        if bytes.len() < prefix_size {
            return Err(BeBytesError::InsufficientData {
                expected: prefix_size,
                actual: bytes.len(),
            });
        }

        let length = T::from_le_bytes(&bytes[..prefix_size]).to_usize();
        let total_size = prefix_size + length;

        if bytes.len() < total_size {
            return Err(BeBytesError::InsufficientData {
                expected: total_size,
                actual: bytes.len(),
            });
        }

        let string_data = &bytes[prefix_size..total_size];

        // Validate UTF-8
        if core::str::from_utf8(string_data).is_err() {
            return Err(BeBytesError::InvalidDiscriminant {
                value: 0, // Not really a discriminant, but reusing error type
                type_name: "VarString (invalid UTF-8)",
            });
        }

        Ok((
            Self {
                data: string_data.to_vec(),
                _phantom: core::marker::PhantomData,
            },
            total_size,
        ))
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
