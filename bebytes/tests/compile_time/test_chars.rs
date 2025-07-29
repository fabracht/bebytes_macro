// Test that char is now supported in bit fields - this should compile successfully
use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct CharBitField {
    #[bits(1)]
    first: u8,
    #[bits(6)]
    second: char,  // Now supported!
    #[bits(1)]
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct CharPrimitive {
    ch: char,  // Primitive char support
}

fn main() {
    // These should all work now
    let _bit_field = CharBitField {
        first: 1,
        second: 'A',
        fourth: 0,
    };
    
    let _primitive = CharPrimitive {
        ch: '🦀',
    };
}
