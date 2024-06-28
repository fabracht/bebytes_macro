use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
use core::fmt::Write;
#[cfg(feature = "std")]
use std::fmt::Write;

#[derive(BeBytes)]
struct UnsupportedStruct(u8, u16, u32);

fn main() {}
