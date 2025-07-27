use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct F64 {
    #[bits(1)]
    first: u8,
    #[bits(62)]
    second: f64,
    #[bits(1)]
    fourth: u8,
}

fn main() {}
