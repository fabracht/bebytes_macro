use bebytes::*;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes)]
struct UnsupportedStruct(u8, u16, u32);

fn main() {}
