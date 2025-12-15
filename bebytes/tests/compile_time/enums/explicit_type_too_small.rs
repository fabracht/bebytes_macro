use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
#[bebytes(flags(u8))]
enum TooSmall {
    Flag1 = 1,
    Flag2 = 256,
}

fn main() {}
