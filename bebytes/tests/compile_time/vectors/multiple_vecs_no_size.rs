// This test verifies that having multiple Vec fields without proper size
// specification is rejected at compile time.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct MultipleVecsNoSize {
    // Error: Multiple Vec fields without size specification
    data1: Vec<u8>,
    data2: Vec<u8>,
}

fn main() {}