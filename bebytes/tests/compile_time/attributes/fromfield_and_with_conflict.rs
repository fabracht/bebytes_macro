// This test verifies that #[FromField] and #[With(size())] cannot be used
// together as they both specify the size source.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct FromFieldAndWithConflict {
    length: u16,
    // Error: Cannot use both #[FromField] and #[With(size())]
    #[FromField(length)]
    #[With(size(10))]
    data: Vec<u8>,
}

fn main() {}