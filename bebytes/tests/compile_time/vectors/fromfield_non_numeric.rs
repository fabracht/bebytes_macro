// This test verifies that #[FromField] cannot reference non-numeric fields
// since vector size must be a number.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct FromFieldNonNumeric {
    name: String,
    // Error: 'name' is not a numeric field
    #[FromField(name)]
    data: Vec<u8>,
}

fn main() {}