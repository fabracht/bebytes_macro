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
