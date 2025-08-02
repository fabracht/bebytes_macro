// This test verifies that #[bits] attribute cannot be used on non-numeric types.
// Bit fields only make sense for integer types.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct BitsOnNonNumeric {
    valid: u8,
    // Error: Cannot use #[bits] on String type
    #[bits(4)]
    invalid: String,
    #[bits(4)]
    padding: u8,
}

fn main() {}