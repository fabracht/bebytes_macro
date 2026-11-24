use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct ByteBoundary {
    #[bits(4)]
    nibble: u8,
    #[UntilMarker(0xFF)]
    after: Vec<u8>,
}

fn main() {}
