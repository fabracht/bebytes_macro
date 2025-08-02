// This test verifies that bit fields cannot exceed the size of their underlying type.
// For example, u8 can only hold 8 bits maximum.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct ExceedsTypeSize {
    // Error: u8 can only hold 8 bits, not 9
    #[bits(9)]
    too_many_bits: u8,
    #[bits(7)]
    padding: u8,
}

fn main() {}