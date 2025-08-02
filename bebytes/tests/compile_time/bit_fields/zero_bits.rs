// This test verifies that zero-bit fields are rejected at compile time.
// A field with zero bits makes no sense and should be caught early.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct ZeroBits {
    #[bits(8)]
    valid: u8,
    // Error: Zero bits is not allowed
    #[bits(0)]
    invalid: u8,
}

fn main() {}