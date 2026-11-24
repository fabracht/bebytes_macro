use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct ByteBoundary {
    #[bits(4)]
    nibble: u8,
    #[With(size(2))]
    after: String,
}

fn main() {}
