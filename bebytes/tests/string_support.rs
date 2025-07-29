use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[test]
fn test_string_fixed_size() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct FixedMessage {
        id: u32,
        #[With(size(10))]
        name: String,
        priority: u8,
    }

    let msg = FixedMessage {
        id: 42,
        name: "Alice     ".to_string(), // Padded to 10 bytes
        priority: 1,
    };

    let bytes = msg.to_be_bytes();
    let (decoded, _) = FixedMessage::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(decoded.id, 42);
    assert_eq!(decoded.name, "Alice     ");
    assert_eq!(decoded.priority, 1);
}

#[test]
fn test_string_from_field() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct VarMessage {
        id: u32,
        name_len: u8,
        #[FromField(name_len)]
        name: String,
        priority: u8,
    }

    let msg = VarMessage {
        id: 42,
        name_len: 5,
        name: "Alice".to_string(),
        priority: 1,
    };

    let bytes = msg.to_be_bytes();
    let (decoded, _) = VarMessage::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(decoded, msg);
}

#[test]
fn test_string_last_field() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Message {
        id: u32,
        content: String,
    }

    let msg = Message {
        id: 42,
        content: "Hello, world! This is a test message.".to_string(),
    };

    let bytes = msg.to_be_bytes();
    let (decoded, _) = Message::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(decoded, msg);
}

#[test]
fn test_string_unicode() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct UnicodeMessage {
        id: u16,
        content: String,
    }

    let msg = UnicodeMessage {
        id: 123,
        content: "Hello 世界! 🦀".to_string(),
    };

    let bytes = msg.to_be_bytes();
    let (decoded, _) = UnicodeMessage::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(decoded, msg);
}

#[test]
fn test_string_empty() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct EmptyMessage {
        id: u32,
        content: String,
    }

    let msg = EmptyMessage {
        id: 42,
        content: String::new(),
    };

    let bytes = msg.to_be_bytes();
    let (decoded, _) = EmptyMessage::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(decoded, msg);
}

#[test]
fn test_string_invalid_utf8() {
    #[derive(BeBytes)]
    struct Message {
        content: String,
    }

    // Create bytes with invalid UTF-8
    let bytes = vec![0xFF, 0xFE, 0xFD];

    let result = Message::try_from_be_bytes(&bytes);
    assert!(result.is_err());

    match result {
        Err(bebytes::BeBytesError::InvalidDiscriminant { type_name, .. }) => {
            assert_eq!(type_name, "String (invalid UTF-8)");
        }
        _ => panic!("Expected InvalidDiscriminant error for UTF-8"),
    }
}

#[test]
fn test_string_fixed_size_padding() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct PaddedMessage {
        #[With(size(10))]
        name: String,
        id: u32,
    }

    // Test that fixed-size strings must match exactly
    let msg = PaddedMessage {
        name: "Bob".to_string(), // 3 bytes, but field expects 10
        id: 42,
    };

    // This should panic during serialization
    #[cfg(feature = "std")]
    let result = std::panic::catch_unwind(|| msg.to_be_bytes());

    #[cfg(feature = "std")]
    assert!(result.is_err());
}

#[test]
fn test_string_nested_field_access() {
    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct Header {
        version: u8,
        name_len: u16,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct Packet {
        header: Header,
        #[FromField(header.name_len)]
        name: String,
        data: Vec<u8>,
    }

    let packet = Packet {
        header: Header {
            version: 1,
            name_len: 7,
        },
        name: "Example".to_string(),
        data: vec![1, 2, 3, 4],
    };

    let bytes = packet.to_be_bytes();
    let (decoded, _) = Packet::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(decoded, packet);
}

#[test]
fn test_string_endianness() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Message {
        len: u16,
        #[FromField(len)]
        content: String,
    }

    let msg = Message {
        len: 4,
        content: "test".to_string(),
    };

    let be_bytes = msg.to_be_bytes();
    let le_bytes = msg.to_le_bytes();

    // The length field should differ in endianness
    assert_eq!(be_bytes[0..2], [0, 4]); // big-endian u16
    assert_eq!(le_bytes[0..2], [4, 0]); // little-endian u16

    // But the string content should be the same
    assert_eq!(&be_bytes[2..], b"test");
    assert_eq!(&le_bytes[2..], b"test");

    let (decoded_be, _) = Message::try_from_be_bytes(&be_bytes).unwrap();
    let (decoded_le, _) = Message::try_from_le_bytes(&le_bytes).unwrap();

    assert_eq!(decoded_be, msg);
    assert_eq!(decoded_le, msg);
}

#[test]
fn test_multiple_strings() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct MultiString {
        id: u32,
        #[With(size(5))]
        name: String,
        desc_len: u8,
        #[FromField(desc_len)]
        description: String,
        notes: String, // Last field, unbounded
    }

    let msg = MultiString {
        id: 100,
        name: "Alice".to_string(),
        desc_len: 11,
        description: "Test person".to_string(),
        notes: "Additional notes here".to_string(),
    };

    let bytes = msg.to_be_bytes();
    let (decoded, _) = MultiString::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(decoded, msg);
}
