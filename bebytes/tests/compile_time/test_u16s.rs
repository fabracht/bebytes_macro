use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
use core::fmt::Write;
#[cfg(feature = "std")]
use std::fmt::Write;

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
struct U16 {
    #[U8(size(1), pos(0))]
    first: u8,
    #[U8(size(14), pos(1))]
    second: u16,
    #[U8(size(1), pos(15))]
    fourth: u8,
}

fn main() {}
