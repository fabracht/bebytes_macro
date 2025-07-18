use bebytes::BeBytes;

#[derive(BeBytes)]
enum InvalidEnum {
    A = 0,
    B = 256, // This exceeds u8 range
    C = 1000, // This also exceeds u8 range
}

fn main() {}