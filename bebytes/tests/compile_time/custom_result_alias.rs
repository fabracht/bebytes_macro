// This test verifies that BeBytes works correctly with custom Result type aliases
// like those found in the MQTT library

use bebytes::BeBytes;

// Define a custom Result type alias similar to MQTT library
pub type Result<T> = std::result::Result<T, MyError>;

#[derive(Debug)]
pub struct MyError {
    message: String,
}

// This should compile successfully
#[derive(BeBytes, Debug, PartialEq)]
pub struct PacketHeader {
    #[bits(4)]
    packet_type: u8,
    #[bits(4)]
    flags: u8,
    length: u16,
}

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
pub enum PacketType {
    Connect = 1,
    ConnAck = 2,
    Publish = 3,
    PubAck = 4,
}

fn main() {
    // Test that we can use the generated code
    let header = PacketHeader {
        packet_type: 3,
        flags: 0,
        length: 1024,
    };
    
    let bytes = header.to_be_bytes();
    let (decoded, _) = PacketHeader::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(header, decoded);
    
    // Test enum
    let packet_type = PacketType::Publish;
    let bytes = packet_type.to_be_bytes();
    let (decoded, _) = PacketType::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(packet_type, decoded);
    
    // Test TryFrom
    use std::convert::TryFrom;
    let result = PacketType::try_from(3u8);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PacketType::Publish);
    
    // This should return an error but compile fine
    let result = PacketType::try_from(99u8);
    assert!(result.is_err());
}