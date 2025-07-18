use bebytes::BeBytes;

// This should fail because 3 is not a power of 2
#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[bebytes(flags)]
#[repr(u8)]
enum InvalidFlags {
    Flag1 = 1,
    Flag2 = 2,
    Flag3 = 3, // Not a power of 2!
    Flag4 = 4,
}

fn main() {}