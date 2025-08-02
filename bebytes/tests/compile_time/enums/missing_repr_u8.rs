// This test verifies that enums without #[repr(u8)] are rejected
// since BeBytes requires explicit representation.

use bebytes::BeBytes;

// Error: Missing #[repr(u8)] attribute
#[derive(BeBytes, Debug, PartialEq)]
enum MissingRepr {
    A = 0,
    B = 1,
    C = 2,
}

fn main() {}