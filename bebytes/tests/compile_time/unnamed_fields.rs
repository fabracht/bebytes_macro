use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
use core::fmt::Write;

#[derive(BeBytes)]
enum UnsupportedEnum {
    A,
    B,
}

fn main() {}
