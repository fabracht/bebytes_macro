use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
use core::fmt::Write;
#[cfg(feature = "std")]
use std::fmt::Write;

#[derive(BeBytes, Debug, PartialEq)]
pub struct Optional {
    pub optional_number: Option<i32>,
}

fn main() {}
