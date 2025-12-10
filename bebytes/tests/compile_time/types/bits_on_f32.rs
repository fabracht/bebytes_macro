use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct F32WithBits {
    #[bits(1)]
    first: u8,
    #[bits(30)]
    second: f32,
    #[bits(1)]
    fourth: u8,
}

fn main() {}
