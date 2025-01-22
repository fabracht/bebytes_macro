use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use core::fmt::Write;

#[derive(BeBytes, Debug, PartialEq)]
struct Char {
    #[U8(size(4), pos(0))]
    first: u8,
    #[U8(size(4), pos(3))]
    second: u8,
}

fn main() {}
