// This test verifies that enums with duplicate discriminant values
// are rejected at compile time.

use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
enum DuplicateDiscriminants {
    A = 0,
    B = 1,
    C = 1, // Error: Duplicate discriminant value
}

fn main() {}