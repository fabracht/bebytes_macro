// This test verifies that #[FromField] referencing a non-existent field
// produces a clear compile-time error.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct FromFieldNonexistent {
    length: u16,
    // Error: 'missing_field' doesn't exist
    #[FromField(missing_field)]
    data: Vec<u8>,
}

fn main() {}