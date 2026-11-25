use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
struct WideArray {
    values: [u16; 4],
}

fn main() {}
