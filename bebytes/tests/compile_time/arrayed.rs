use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
use core::fmt::Write;

#[derive(BeBytes, Debug, PartialEq, Clone)]
pub struct ArrayedStruct {
    pub key_id: [u8; 1],
    pub token: [u8; 2],
    pub client_iv: [u8; 3],
}

fn main() {}
