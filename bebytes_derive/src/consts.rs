pub const PRIMITIVES: [&str; 14] = [
    "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64", "bool",
    "char",
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Endianness {
    Big,
    Little,
}
