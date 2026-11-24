use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct Inner {
    value: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct ByteBoundary {
    #[bits(4)]
    nibble: u8,
    after: Inner,
}

fn main() {}
