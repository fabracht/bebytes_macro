//! Type-safe builder pattern for BeBytes structures
//!
//! This module provides a builder pattern with compile-time state validation
//! to ensure that BeBytes structures are constructed correctly.

use core::marker::PhantomData;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

/// Marker trait for builder states
pub trait BuilderState: private::Sealed {}

/// Initial state - no data written yet
pub struct Empty;

/// State after writing fixed-size data
pub struct HasFixed;

/// State after writing variable-size data (can only add more variable data)
pub struct HasVariable;

/// State ready to build (all required fields present)
pub struct Complete;

impl BuilderState for Empty {}
impl BuilderState for HasFixed {}
impl BuilderState for HasVariable {}
impl BuilderState for Complete {}

/// Type-safe builder for constructing byte sequences
///
/// This builder enforces at compile-time that:
/// - Fixed-size fields are added before variable-size fields
/// - Variable-size fields with size hints are properly configured
/// - The structure is complete before building
///
/// # Example
///
/// ```rust
/// use bebytes::builder::BytesBuilder;
///
/// let data = BytesBuilder::new()
///     .u8(0x42)              // Fixed size
///     .u32_be(0x12345678)    // Fixed size
///     .with_size(10)         // Size hint for next field
///     .bytes(vec![1,2,3])    // Variable size with hint
///     .append_bytes(vec![4,5,6]) // Append more variable data
///     .build_variable();
/// ```
pub struct BytesBuilder<State: BuilderState> {
    buffer: Vec<u8>,
    _state: PhantomData<State>,
}

impl BytesBuilder<Empty> {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            _state: PhantomData,
        }
    }

    /// Create a new builder with pre-allocated capacity
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            _state: PhantomData,
        }
    }
}

impl Default for BytesBuilder<Empty> {
    fn default() -> Self {
        Self::new()
    }
}

// Methods available in Empty or HasFixed states (can add fixed-size data)
impl<S> BytesBuilder<S>
where
    S: BuilderState + AllowsFixed,
{
    /// Add a single byte
    pub fn u8(mut self, value: u8) -> BytesBuilder<HasFixed> {
        self.buffer.push(value);
        BytesBuilder {
            buffer: self.buffer,
            _state: PhantomData,
        }
    }

    /// Add a u16 in big-endian
    pub fn u16_be(mut self, value: u16) -> BytesBuilder<HasFixed> {
        self.buffer.extend_from_slice(&value.to_be_bytes());
        BytesBuilder {
            buffer: self.buffer,
            _state: PhantomData,
        }
    }

    /// Add a u16 in little-endian
    pub fn u16_le(mut self, value: u16) -> BytesBuilder<HasFixed> {
        self.buffer.extend_from_slice(&value.to_le_bytes());
        BytesBuilder {
            buffer: self.buffer,
            _state: PhantomData,
        }
    }

    /// Add a u32 in big-endian
    pub fn u32_be(mut self, value: u32) -> BytesBuilder<HasFixed> {
        self.buffer.extend_from_slice(&value.to_be_bytes());
        BytesBuilder {
            buffer: self.buffer,
            _state: PhantomData,
        }
    }

    /// Add a u32 in little-endian
    pub fn u32_le(mut self, value: u32) -> BytesBuilder<HasFixed> {
        self.buffer.extend_from_slice(&value.to_le_bytes());
        BytesBuilder {
            buffer: self.buffer,
            _state: PhantomData,
        }
    }

    /// Add a fixed-size byte array
    pub fn fixed_bytes<const N: usize>(mut self, bytes: [u8; N]) -> BytesBuilder<HasFixed> {
        self.buffer.extend_from_slice(&bytes);
        BytesBuilder {
            buffer: self.buffer,
            _state: PhantomData,
        }
    }
}

/// Builder with size context for the next variable field
pub struct SizedBuilder<State: BuilderState> {
    buffer: Vec<u8>,
    size: usize,
    _state: PhantomData<State>,
}

// Transition to sized builder for variable-length fields
impl BytesBuilder<HasFixed> {
    /// Specify the size for the next variable-length field
    pub fn with_size(self, size: usize) -> SizedBuilder<HasFixed> {
        SizedBuilder {
            buffer: self.buffer,
            size,
            _state: PhantomData,
        }
    }

    /// Add remaining bytes (transitions to variable state)
    pub fn remaining_bytes(mut self, bytes: Vec<u8>) -> BytesBuilder<HasVariable> {
        self.buffer.extend_from_slice(&bytes);
        BytesBuilder {
            buffer: self.buffer,
            _state: PhantomData,
        }
    }
}

impl<S: BuilderState> SizedBuilder<S> {
    /// Add bytes with the previously specified size
    pub fn bytes(mut self, bytes: Vec<u8>) -> BytesBuilder<HasVariable> {
        let actual_size = bytes.len().min(self.size);
        self.buffer.extend_from_slice(&bytes[..actual_size]);

        // Pad if necessary
        if actual_size < self.size {
            self.buffer
                .resize(self.buffer.len() + (self.size - actual_size), 0);
        }

        BytesBuilder {
            buffer: self.buffer,
            _state: PhantomData,
        }
    }

    /// Add a string with the previously specified size
    #[cfg(feature = "std")]
    pub fn string(self, s: String) -> BytesBuilder<HasVariable> {
        self.bytes(s.into_bytes())
    }

    #[cfg(not(feature = "std"))]
    pub fn string(self, s: alloc::string::String) -> BytesBuilder<HasVariable> {
        self.bytes(s.into_bytes())
    }
}

// Methods for variable state
impl BytesBuilder<HasVariable> {
    /// Add more variable bytes
    pub fn append_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.buffer.extend_from_slice(&bytes);
        self
    }

    /// Mark the builder as complete
    pub fn complete(self) -> BytesBuilder<Complete> {
        BytesBuilder {
            buffer: self.buffer,
            _state: PhantomData,
        }
    }
}

// Build method only available in Complete state
impl BytesBuilder<Complete> {
    /// Build the final byte vector
    #[must_use]
    pub fn build(self) -> Vec<u8> {
        self.buffer
    }
}

// Convenience build for simple cases
impl BytesBuilder<HasFixed> {
    /// Build directly from fixed-only state
    #[must_use]
    pub fn build_fixed(self) -> Vec<u8> {
        self.buffer
    }
}

impl BytesBuilder<HasVariable> {
    /// Build directly from variable state
    #[must_use]
    pub fn build_variable(self) -> Vec<u8> {
        self.buffer
    }
}

/// Marker trait for states that allow adding fixed-size data
pub trait AllowsFixed: BuilderState {}

impl AllowsFixed for Empty {}
impl AllowsFixed for HasFixed {}

// Private module to seal the BuilderState trait
mod private {
    pub trait Sealed {}
    impl Sealed for super::Empty {}
    impl Sealed for super::HasFixed {}
    impl Sealed for super::HasVariable {}
    impl Sealed for super::Complete {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_only_builder() {
        let data = BytesBuilder::new()
            .u8(0x42)
            .u16_be(0x1234)
            .u32_be(0xDEADBEEF)
            .build_fixed();

        assert_eq!(data, vec![0x42, 0x12, 0x34, 0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_mixed_builder() {
        let data = BytesBuilder::new()
            .u8(0x01)
            .u32_be(0x12345678)
            .with_size(5)
            .bytes(vec![0xAA, 0xBB, 0xCC]) // Only 3 bytes, will be padded
            .build_variable();

        assert_eq!(
            data,
            vec![0x01, 0x12, 0x34, 0x56, 0x78, 0xAA, 0xBB, 0xCC, 0x00, 0x00]
        );
    }

    #[test]
    fn test_remaining_bytes() {
        let data = BytesBuilder::new()
            .u8(0xFF)
            .remaining_bytes(vec![0x11, 0x22, 0x33])
            .append_bytes(vec![0x44, 0x55])
            .build_variable();

        assert_eq!(data, vec![0xFF, 0x11, 0x22, 0x33, 0x44, 0x55]);
    }

    // This would fail to compile, demonstrating type safety:
    // fn test_invalid_order() {
    //     let data = BytesBuilder::new()
    //         .remaining_bytes(vec![1, 2, 3])
    //         .u8(0x42)  // ERROR: Can't add fixed after variable
    //         .build();
    // }
}
