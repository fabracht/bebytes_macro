use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use core::fmt::Write;

#[derive(BeBytes, Debug, PartialEq)]
struct IncompleteByte {
    #[bits(3)]
    first: u8,
    #[bits(4)]
    second: u8,
    // Total: 7 bits - not a complete byte!
}

fn main() {}