#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
pub use alloc::borrow::ToOwned;
#[cfg(not(feature = "std"))]
use core::convert::Infallible;

pub use bebytes_derive::BeBytes;

pub trait BeBytes {
    fn field_size() -> usize;

    // Big-endian methods
    #[cfg(feature = "std")]
    fn to_be_bytes(&self) -> std::vec::Vec<u8>;

    #[cfg(not(feature = "std"))]
    fn to_be_bytes(&self) -> alloc::vec::Vec<u8>;

    #[cfg(feature = "std")]
    fn try_from_be_bytes(
        bytes: &'_ [u8],
    ) -> core::result::Result<(Self, usize), std::boxed::Box<dyn std::error::Error>>
    where
        Self: Sized;

    #[cfg(not(feature = "std"))]
    fn try_from_be_bytes(bytes: &'_ [u8]) -> core::result::Result<(Self, usize), Infallible>
    where
        Self: Sized;

    // Little-endian methods
    #[cfg(feature = "std")]
    fn to_le_bytes(&self) -> std::vec::Vec<u8>;

    #[cfg(not(feature = "std"))]
    fn to_le_bytes(&self) -> alloc::vec::Vec<u8>;

    #[cfg(feature = "std")]
    fn try_from_le_bytes(
        bytes: &'_ [u8],
    ) -> core::result::Result<(Self, usize), std::boxed::Box<dyn std::error::Error>>
    where
        Self: Sized;

    #[cfg(not(feature = "std"))]
    fn try_from_le_bytes(bytes: &'_ [u8]) -> core::result::Result<(Self, usize), Infallible>
    where
        Self: Sized;
}

pub trait BeBytesWith {
    fn try_from_be_bytes_with_sizes(
        bytes: &[u8],
        sizes: &std::collections::HashMap<&'static str, usize>,
    ) -> core::result::Result<(Self, usize), std::boxed::Box<dyn std::error::Error>>
    where
        Self: Sized + BeBytesMetadata;

    /// Attempts to parse this type from a little-endian byte slice with external size information.
    /// Used for structs that have fields requiring size information from other structs.
    fn try_from_le_bytes_with_sizes(
        bytes: &[u8],
        sizes: &std::collections::HashMap<&'static str, usize>,
    ) -> core::result::Result<(Self, usize), std::boxed::Box<dyn std::error::Error>>
    where
        Self: Sized + BeBytesMetadata;
}

pub trait BeBytesMetadata {
    fn for_field_mappings() -> std::collections::HashMap<&'static str, &'static str> {
        std::collections::HashMap::new()
    }

    fn requires_external_sizes() -> std::collections::HashSet<&'static str> {
        std::collections::HashSet::new()
    }
}
