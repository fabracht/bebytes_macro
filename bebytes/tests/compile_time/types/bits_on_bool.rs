use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct BoolWithBits {
    #[bits(1)]
    first: u8,
    #[bits(6)]
    second: bool,
    #[bits(1)]
    fourth: u8,
}

fn main() {}
