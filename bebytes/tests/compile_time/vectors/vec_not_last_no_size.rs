// This test verifies that a Vec field without size specification
// must be the last field in the struct.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct VecNotLastNoSize {
    // Error: Vec without size must be last field
    data: Vec<u8>,
    footer: u32,
}

fn main() {}