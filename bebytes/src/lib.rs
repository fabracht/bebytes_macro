#![no_std]

#[cfg(feature = "std")]
extern crate std;
#[cfg(feature = "std")]
use std::boxed::Box;
#[cfg(feature = "std")]
use std::error::Error;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use core::convert::Infallible;

pub use bebytes_derive::BeBytes;

pub trait BeBytes {
    fn field_size() -> usize;

    fn to_be_bytes(&self) -> Vec<u8>;

    #[cfg(feature = "std")]
    fn try_from_be_bytes(bytes: &'_ [u8]) -> core::result::Result<(Self, usize), Box<dyn Error>>
    where
        Self: Sized;

    #[cfg(not(feature = "std"))]
    fn try_from_be_bytes(bytes: &'_ [u8]) -> core::result::Result<(Self, usize), Infallible>
    where
        Self: Sized;
}
