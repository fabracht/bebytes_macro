use bebytes::*;
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use core::fmt::Write;

#[derive(BeBytes, Debug, PartialEq)]
struct U8 {
    #[U8(size(1), pos(0))]
    first: u8,
    #[U8(size(3), pos(1))]
    second: u8,
    #[U8(size(4), pos(4))]
    third: u8,
    fourth: u8,
}

fn main() {}
