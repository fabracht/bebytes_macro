use bebytes::BeBytes;

// This should succeed - zero is allowed in flag enums
#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[bebytes(flags)]
#[repr(u8)]
enum FlagsWithZero {
    None = 0,    // Zero is allowed
    Flag1 = 1,
    Flag2 = 2,
    Flag4 = 4,
}

fn main() {
    // This should compile fine
    let _flags = FlagsWithZero::Flag1 | FlagsWithZero::Flag2;
}