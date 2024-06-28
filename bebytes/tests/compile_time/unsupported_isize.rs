use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
use core::fmt::Write;
#[cfg(feature = "std")]
use std::fmt::Write;

#[derive(BeBytes, Debug, PartialEq)]
struct ISize {
    #[U8(size(1), pos(0))]
    first: u8,
    #[U8(size(62), pos(1))]
    second: isize,
    #[U8(size(1), pos(63))]
    fourth: u8,
}

fn main() {}
