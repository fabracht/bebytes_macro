use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
pub struct NestedStruct {
    pub dummy_struct: DummyStruct,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
pub struct DummyStruct {
    pub dummy0: [u8; 2],
    #[U8(size(1), pos(0))]
    pub dummy1: u8,
    #[U8(size(7), pos(1))]
    pub dummy2: u8,
}

fn main() {}
