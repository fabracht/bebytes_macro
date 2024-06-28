use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
struct Char {
    #[U8(size(4), pos(0))]
    first: u8,
    #[U8(size(4), pos(3))]
    second: u8,
}

fn main() {}
