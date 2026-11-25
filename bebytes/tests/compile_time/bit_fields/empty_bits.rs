use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
struct EmptyBits {
    #[bits()]
    value: u8,
}

fn main() {}
