// This test verifies that multiple endianness attributes on the same field
// are detected and rejected.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct MultipleEndianAttrs {
    // Error: Conflicting endianness attributes
    #[bebytes(big_endian)]
    #[bebytes(little_endian)]
    value: u32,
}

fn main() {}