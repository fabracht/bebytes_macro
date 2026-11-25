use bebytes::BeBytes;

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct Inner {
    id: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct Outer {
    items: Vec<Inner>,
    tail: u8,
}

fn main() {}
