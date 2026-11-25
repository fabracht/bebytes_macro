use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
struct MarkerOnPrimitive {
    #[UntilMarker(0xFF)]
    value: u8,
}

fn main() {}
