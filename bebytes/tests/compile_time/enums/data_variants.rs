// This test verifies that enums with data variants (non-unit variants)
// are rejected since BeBytes only supports simple enums.

use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
enum DataVariants {
    Unit = 0,
    // Error: Data variants not supported
    Tuple(u32),
    Struct { field: u16 },
}

fn main() {}
