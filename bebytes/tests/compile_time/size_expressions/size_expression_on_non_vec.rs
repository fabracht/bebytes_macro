use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
struct SizeExpressionOnPrimitive {
    count: u8,
    #[bebytes(size = "count * 2")]
    value: u32,
}

fn main() {}
