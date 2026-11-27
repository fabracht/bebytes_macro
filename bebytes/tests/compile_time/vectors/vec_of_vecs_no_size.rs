use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
struct NoSegmentCount {
    #[UntilMarker(0xFF)]
    sections: Vec<Vec<u8>>,
}

fn main() {}
