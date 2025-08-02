// This test verifies that multiple #[bits] attributes on the same field
// are detected and rejected.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct MultipleBitsAttributes {
    // Error: Multiple #[bits] attributes on same field
    #[bits(4)]
    #[bits(4)]
    field: u8,
}

fn main() {}