// This test verifies that #[bits] and #[bebytes(size)] cannot be used
// on the same field as they represent conflicting sizing mechanisms.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct BitsAndSizeConflict {
    length: u16,
    // Error: Cannot use both #[bits] and #[bebytes(size)] on same field
    #[bits(4)]
    #[bebytes(size = "length")]
    data: Vec<u8>,
    #[bits(4)]
    padding: u8,
}

fn main() {}