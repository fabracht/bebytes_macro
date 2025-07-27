use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]

#[derive(BeBytes, Debug, PartialEq)]
struct Char {
    #[bits(1)]
    first: u8,
    #[bits(6)]
    second: char,
    #[bits(1)]
    fourth: u8,
}

fn main() {}
