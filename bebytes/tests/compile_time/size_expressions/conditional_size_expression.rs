use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
struct Conditional {
    flag: u8,
    #[With(size(if flag == 1 { 4 } else { 8 }))]
    data: Vec<u8>,
}

fn main() {}
