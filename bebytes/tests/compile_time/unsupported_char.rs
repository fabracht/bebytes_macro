use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use core::fmt::Write;

#[derive(BeBytes, Debug, PartialEq)]
struct Char {
    #[U8(size(1), pos(0))]
    first: u8,
    #[U8(size(6), pos(1))]
    second: char,
    #[U8(size(1), pos(7))]
    fourth: u8,
}

fn main() {}
