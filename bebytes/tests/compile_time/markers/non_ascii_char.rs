use bebytes::BeBytes;

#[derive(BeBytes)]
struct NonAsciiMarker {
    header: u8,
    #[UntilMarker('€')]  // Euro symbol is not ASCII
    data: Vec<u8>,
    footer: u8,
}

fn main() {}