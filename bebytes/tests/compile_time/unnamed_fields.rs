use bebytes::*;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes)]
enum UnsupportedEnum {
    A,
    B,
}

fn main() {}
