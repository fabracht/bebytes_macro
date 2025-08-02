// This test verifies that referencing a non-existent field in a size expression
// produces a clear compile-time error.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct InvalidSizeRef {
    length: u16,
    // Error: 'nonexistent' field doesn't exist
    #[bebytes(size = "nonexistent")]
    data: Vec<u8>,
}

fn main() {}