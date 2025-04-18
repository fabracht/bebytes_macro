#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
pub use alloc::borrow::ToOwned;
#[cfg(not(feature = "std"))]
use core::convert::Infallible; // Keep this if used by implementations

pub use bebytes_derive::BeBytes;

/// A trait for converting types to and from byte representations in big-endian or little-endian format.
pub trait BeBytes {
    /// Returns the fixed size of the type in bytes, if known at compile time.
    /// Implementations for dynamically sized types might return 0 or handle size differently.
    fn field_size() -> usize; // Added basic doc

    // Big-endian methods

    /// Converts the value to its big-endian byte representation.
    #[cfg(feature = "std")]
    fn to_be_bytes(&self) -> std::vec::Vec<u8>;

    /// Converts the value to its big-endian byte representation.
    #[cfg(not(feature = "std"))]
    fn to_be_bytes(&self) -> alloc::vec::Vec<u8>;

    /// Attempts to parse an instance of `Self` from the beginning of a big-endian byte slice.
    ///
    /// Returns a tuple containing the parsed instance and the number of bytes consumed
    /// on success.
    ///
    /// # Errors
    ///
    /// This function will return an error boxed in `std::boxed::Box<dyn std::error::Error>` if:
    /// * The byte slice is too short to contain a valid representation of `Self`.
    /// * The bytes represent an invalid value that cannot be parsed into `Self`.
    /// * An error occurs during the parsing of nested types, if `Self` is a composite type.
    #[cfg(feature = "std")]
    fn try_from_be_bytes(
        bytes: &'_ [u8],
    ) -> core::result::Result<(Self, usize), std::boxed::Box<dyn std::error::Error>>
    where
        Self: Sized;

    /// Attempts to parse an instance of `Self` from the beginning of a big-endian byte slice.
    ///
    /// Returns a tuple containing the parsed instance and the number of bytes consumed.
    /// Note: This `no_std` version uses `Infallible` as the error type, implying it does not return `Err`.
    /// Check specific implementations for behavior on invalid input (e.g., panic, preconditions).
    #[cfg(not(feature = "std"))]
    fn try_from_be_bytes(bytes: &'_ [u8]) -> core::result::Result<(Self, usize), Infallible>
    // Assuming Infallible here too based on pattern
    where
        Self: Sized;

    // Little-endian methods

    /// Converts the value to its little-endian byte representation.
    #[cfg(feature = "std")]
    fn to_le_bytes(&self) -> std::vec::Vec<u8>;

    /// Converts the value to its little-endian byte representation.
    #[cfg(not(feature = "std"))]
    fn to_le_bytes(&self) -> alloc::vec::Vec<u8>;

    /// Attempts to parse an instance of `Self` from the beginning of a little-endian byte slice.
    ///
    /// Returns a tuple containing the parsed instance and the number of bytes consumed
    /// on success.
    ///
    /// # Errors
    ///
    /// This function will return an error boxed in `std::boxed::Box<dyn std::error::Error>` if:
    /// * The byte slice is too short to contain a valid representation of `Self`.
    /// * The bytes represent an invalid value that cannot be parsed into `Self`.
    /// * An error occurs during the parsing of nested types, if `Self` is a composite type.
    #[cfg(feature = "std")]
    fn try_from_le_bytes(
        bytes: &'_ [u8],
    ) -> core::result::Result<(Self, usize), std::boxed::Box<dyn std::error::Error>>
    where
        Self: Sized;

    /// Attempts to parse an instance of `Self` from the beginning of a little-endian byte slice.
    ///
    /// Returns a tuple containing the parsed instance and the number of bytes consumed.
    /// Note: This `no_std` version uses `Infallible` as the error type, implying it does not return `Err`.
    /// Check specific implementations for behavior on invalid input (e.g., panic, preconditions).
    #[cfg(not(feature = "std"))]
    fn try_from_le_bytes(bytes: &'_ [u8]) -> core::result::Result<(Self, usize), Infallible>
    where
        Self: Sized;
}
