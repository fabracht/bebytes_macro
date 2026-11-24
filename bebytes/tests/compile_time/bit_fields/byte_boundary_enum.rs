use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
enum Kind {
    First = 0,
    Second = 1,
}

#[derive(BeBytes, Debug, PartialEq)]
struct ByteBoundary {
    #[bits(4)]
    nibble: u8,
    after: Kind,
}

fn main() {}
