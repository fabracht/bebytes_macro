use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use core::fmt::Write;

#[derive(BeBytes, Debug, PartialEq)]
struct ISize {
    #[bits(1)]
    first: u8,
    #[bits(62)]
    second: isize,
    #[bits(1)]
    fourth: u8,
}

fn main() {}
