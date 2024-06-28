use bebytes::BeBytes;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use core::fmt::Write;
#[cfg(feature = "std")]
use std::fmt::Write;

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct U32 {
    #[U8(size(1), pos(0))]
    first: u8,
    #[U8(size(30), pos(1))]
    second: u32,
    #[U8(size(1), pos(31))]
    fourth: u8,
}

fn main() {}
