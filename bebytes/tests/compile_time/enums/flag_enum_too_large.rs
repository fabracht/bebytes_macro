// This test verifies that flag enums with values > 255 are rejected
// since they must fit in a u8.

use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
#[bebytes(flags)]
enum FlagEnumTooLarge {
    Flag1 = 1,
    Flag2 = 2,
    Flag3 = 256, // Error: Value too large for u8
}

fn main() {}