// This test verifies that invalid operators in size expressions
// produce a compile-time error.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct InvalidOperator {
    length: u16,
    // Error: Bitwise operators not supported in size expressions
    #[bebytes(size = "length & 0xFF")]
    data: Vec<u8>,
}

fn main() {}