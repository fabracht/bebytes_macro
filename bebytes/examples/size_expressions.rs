use bebytes::BeBytes as _;
use bebytes_derive::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
struct SimpleMessage {
    count: u8,
    #[With(size(count * 4))]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct StringMessage {
    length: u8,
    #[With(size(length))]
    message: String,
}

fn main() {
    // Test basic size expression
    let msg = SimpleMessage {
        count: 3,
        data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    };

    let bytes = msg.to_be_bytes();
    let (parsed, _) = SimpleMessage::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(msg, parsed);
    println!("Simple size expression test passed!");

    // Test string size expression
    let str_msg = StringMessage {
        length: 5,
        message: "Hello".to_string(),
    };

    let bytes = str_msg.to_be_bytes();
    let (parsed, _) = StringMessage::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(str_msg, parsed);
    println!("String size expression test passed!");

    println!("All size expression tests passed!");
}
