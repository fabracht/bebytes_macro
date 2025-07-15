use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use core::fmt::Write;

#[derive(BeBytes, Debug, PartialEq)]
struct U8 {
    #[bits(1)]
    first: u8,
    #[bits(3)]
    second: u8,
    #[bits(4)]
    third: u8,
    fourth: u8,
}

fn main() {}
