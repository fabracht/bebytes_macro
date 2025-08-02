// This test verifies that division by zero in size expressions
// is caught at compile time.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct DivisionByZero {
    count: u16,
    // Error: Division by zero in size expression
    #[bebytes(size = "count / 0")]
    data: Vec<u8>,
}

fn main() {}