use bebytes::{BeBytes, VarString8, VarString16, VarString32};
#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[test]
fn test_var_string8_basic_functionality() {
    let vs = VarString8::from_str("Hello");
    
    assert_eq!(vs.len(), 5);
    assert_eq!(vs.as_str(), Some("Hello"));
    assert!(!vs.is_empty());
    assert_eq!(vs.as_bytes(), b"Hello");
}

#[test]
fn test_var_string8_serialization() {
    let vs = VarString8::from_str("test");
    
    let bytes = vs.to_be_bytes();
    // Should be: [4, b't', b'e', b's', b't'] = 5 bytes total
    assert_eq!(bytes, vec![4, b't', b'e', b's', b't']);
    
    let (decoded, bytes_read) = VarString8::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(bytes_read, 5);
    assert_eq!(decoded.as_str(), Some("test"));
    assert_eq!(decoded, vs);
}

#[test]
fn test_var_string8_empty() {
    let vs = VarString8::new();
    
    assert_eq!(vs.len(), 0);
    assert_eq!(vs.as_str(), Some(""));
    assert!(vs.is_empty());
    
    let bytes = vs.to_be_bytes();
    assert_eq!(bytes, vec![0]); // Just the length prefix
    
    let (decoded, bytes_read) = VarString8::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(bytes_read, 1);
    assert_eq!(decoded, vs);
}

#[test]
fn test_var_string8_in_struct() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Message {
        id: u32,
        content: VarString8,
        priority: u8,
    }
    
    let msg = Message {
        id: 42,
        content: VarString8::from_str("Hello, world!"),
        priority: 1,
    };
    
    let bytes = msg.to_be_bytes();
    let (decoded, _) = Message::try_from_be_bytes(&bytes).unwrap();
    
    assert_eq!(decoded, msg);
    assert_eq!(decoded.content.as_str(), Some("Hello, world!"));
}

#[test]
fn test_var_string16_large_content() {
    // Create a string larger than u8::MAX
    let large_content = "x".repeat(300);
    let vs = VarString16::from_str(&large_content);
    
    assert_eq!(vs.len(), 300);
    assert_eq!(vs.as_str(), Some(large_content.as_str()));
    
    let bytes = vs.to_be_bytes();
    // Should be: [0x01, 0x2C] (300 in big-endian u16) + 300 bytes of 'x'
    assert_eq!(bytes.len(), 2 + 300);
    assert_eq!(bytes[0..2], [0x01, 0x2C]); // 300 in big-endian
    
    let (decoded, bytes_read) = VarString16::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(bytes_read, 302);
    assert_eq!(decoded.as_str(), Some(large_content.as_str()));
}

#[test]
fn test_var_string32_very_large_content() {
    // Create a string larger than u16::MAX  
    let large_content = "y".repeat(70000);
    let vs = VarString32::from_str(&large_content);
    
    assert_eq!(vs.len(), 70000);
    
    let bytes = vs.to_be_bytes();
    assert_eq!(bytes.len(), 4 + 70000); // u32 prefix + content
    
    let (decoded, bytes_read) = VarString32::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(bytes_read, 70004);
    assert_eq!(decoded.as_str(), Some(large_content.as_str()));
}

#[test]
fn test_var_string_unicode() {
    let vs = VarString8::from_str("Hello 世界! 🦀");
    
    // "Hello 世界! 🦀" = "Hello " (6) + "世" (3) + "界" (3) + "! " (2) + "🦀" (4) = 18 bytes
    assert_eq!(vs.len(), 18);
    assert_eq!(vs.as_str(), Some("Hello 世界! 🦀"));
    
    let bytes = vs.to_be_bytes();
    assert_eq!(bytes[0], 18); // Length prefix
    assert_eq!(bytes.len(), 19); // 1 byte prefix + 18 bytes content
    
    let (decoded, _) = VarString8::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded.as_str(), Some("Hello 世界! 🦀"));
}

#[test]
fn test_var_string_endianness() {
    let vs = VarString16::from_str("test");
    
    let be_bytes = vs.to_be_bytes();
    let le_bytes = vs.to_le_bytes();
    
    // Big-endian: [0, 4, b't', b'e', b's', b't']
    // Little-endian: [4, 0, b't', b'e', b's', b't']
    assert_eq!(be_bytes, vec![0, 4, b't', b'e', b's', b't']);
    assert_eq!(le_bytes, vec![4, 0, b't', b'e', b's', b't']);
    
    let (decoded_be, _) = VarString16::try_from_be_bytes(&be_bytes).unwrap();
    let (decoded_le, _) = VarString16::try_from_le_bytes(&le_bytes).unwrap();
    
    assert_eq!(decoded_be, decoded_le);
    assert_eq!(decoded_be.as_str(), Some("test"));
}

#[test]
fn test_var_string_insufficient_data_for_prefix() {
    // Test with not enough bytes for u16 prefix
    let short_bytes = vec![1]; // Need 2 bytes for u16 prefix
    let result = VarString16::try_from_be_bytes(&short_bytes);
    
    assert!(result.is_err());
    match result {
        Err(bebytes::BeBytesError::InsufficientData { expected, actual }) => {
            assert_eq!(expected, 2);
            assert_eq!(actual, 1);
        }
        _ => panic!("Expected InsufficientData error for prefix"),
    }
}

#[test]
fn test_var_string_insufficient_data_for_content() {
    // Test with prefix indicating more content than available
    let bytes = vec![0, 5, b'h', b'i']; // Says 5 bytes, but only 2 provided
    let result = VarString16::try_from_be_bytes(&bytes);
    
    assert!(result.is_err());
    match result {
        Err(bebytes::BeBytesError::InsufficientData { expected, actual }) => {
            assert_eq!(expected, 7); // 2 bytes prefix + 5 bytes content
            assert_eq!(actual, 4);
        }
        _ => panic!("Expected InsufficientData error for content"),
    }
}

#[test]
fn test_var_string_invalid_utf8() {
    // Create bytes with invalid UTF-8 sequence
    let mut bytes = vec![0, 3]; // 3 bytes of content
    bytes.extend_from_slice(&[0xFF, 0xFE, 0xFD]); // Invalid UTF-8
    
    let result = VarString16::try_from_be_bytes(&bytes);
    
    assert!(result.is_err());
    match result {
        Err(bebytes::BeBytesError::InvalidDiscriminant { type_name, .. }) => {
            assert_eq!(type_name, "VarString (invalid UTF-8)");
        }
        _ => panic!("Expected InvalidDiscriminant error for UTF-8"),
    }
}

#[test]
fn test_var_string_string_too_long_for_prefix() {
    // This should panic when trying to serialize a string too long for u8 prefix
    let long_content = "x".repeat(256); // 256 bytes, too long for u8
    let vs = VarString8::from_str(&long_content);
    
    // This should panic during serialization
    #[cfg(feature = "std")]
    let result = std::panic::catch_unwind(|| {
        vs.to_be_bytes()
    });
    
    #[cfg(not(feature = "std"))]
    let result = Ok(()); // Skip panic test in no_std
    
    #[cfg(feature = "std")]
    assert!(result.is_err());
}

#[test]
fn test_var_string_manipulation() {
    let mut vs = VarString8::from_str("Hello");
    
    vs.push_str(", world");
    assert_eq!(vs.as_str(), Some("Hello, world"));
    assert_eq!(vs.len(), 12);
    
    vs.push('!');
    assert_eq!(vs.as_str(), Some("Hello, world!"));
    assert_eq!(vs.len(), 13);
    
    vs.clear();
    assert_eq!(vs.as_str(), Some(""));
    assert_eq!(vs.len(), 0);
    assert!(vs.is_empty());
}

#[test]
fn test_var_string_from_conversions() {
    // Test From<&str>
    let vs1: VarString8 = "hello".into();
    assert_eq!(vs1.as_str(), Some("hello"));
    
    // Test From<String>
    #[cfg(feature = "std")]
    {
        let vs2: VarString8 = String::from("world").into();
        assert_eq!(vs2.as_str(), Some("world"));
    }
    
    #[cfg(not(feature = "std"))]
    {
        let vs2: VarString8 = alloc::string::String::from("world").into();
        assert_eq!(vs2.as_str(), Some("world"));
    }
}

#[test]
fn test_var_string_display() {
    let vs = VarString8::from_str("test");
    assert_eq!(format!("{}", vs), "test");
    
    let empty_vs = VarString8::new();
    assert_eq!(format!("{}", empty_vs), "");
}

#[test]
fn test_var_string_complex_struct() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct ComplexMessage {
        version: u8,
        sender: VarString8,
        recipient: VarString16,
        subject: VarString8,
        body: VarString32,
        timestamp: u64,
    }
    
    let msg = ComplexMessage {
        version: 1,
        sender: VarString8::from_str("alice"),
        recipient: VarString16::from_str("bob@example.com"),
        subject: VarString8::from_str("Hello"),
        body: VarString32::from_str("This is a test message with some content."),
        timestamp: 1640995200,
    };
    
    let bytes = msg.to_be_bytes();
    let (decoded, _) = ComplexMessage::try_from_be_bytes(&bytes).unwrap();
    
    assert_eq!(decoded, msg);
    assert_eq!(decoded.sender.as_str(), Some("alice"));
    assert_eq!(decoded.recipient.as_str(), Some("bob@example.com"));
    assert_eq!(decoded.subject.as_str(), Some("Hello"));
    assert_eq!(decoded.body.as_str(), Some("This is a test message with some content."));
}

#[test]
fn test_var_string_max_lengths() {
    // Test VarString8 with maximum length (255 bytes)
    let max_content_8 = "x".repeat(255);
    let vs8 = VarString8::from_str(&max_content_8);
    let bytes8 = vs8.to_be_bytes();
    assert_eq!(bytes8[0], 255);
    assert_eq!(bytes8.len(), 256); // 1 byte prefix + 255 content
    
    let (decoded8, _) = VarString8::try_from_be_bytes(&bytes8).unwrap();
    assert_eq!(decoded8.len(), 255);
    
    // Test VarString16 with a moderately large string
    let content_16 = "y".repeat(1000);
    let vs16 = VarString16::from_str(&content_16);
    let bytes16 = vs16.to_be_bytes();
    assert_eq!(bytes16.len(), 1002); // 2 byte prefix + 1000 content
    
    let (decoded16, _) = VarString16::try_from_be_bytes(&bytes16).unwrap();
    assert_eq!(decoded16.len(), 1000);
}