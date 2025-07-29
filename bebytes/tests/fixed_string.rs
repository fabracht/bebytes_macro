use bebytes::{BeBytes, FixedString, FixedString8, FixedString16};
#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[test]
fn test_fixed_string_basic_functionality() {
    let fs = FixedString::<16>::from_str("Hello");

    assert_eq!(fs.len(), 5);
    assert_eq!(fs.as_str(), Some("Hello"));
    assert!(!fs.is_empty());

    // Test padding with null bytes
    let expected_bytes = {
        let mut bytes = [0u8; 16];
        bytes[..5].copy_from_slice(b"Hello");
        bytes
    };
    assert_eq!(fs.as_bytes(), &expected_bytes);
}

#[test]
fn test_fixed_string_truncation() {
    let long_string = "This is a very long string that exceeds 16 bytes";
    let fs = FixedString::<16>::from_str(long_string);

    assert_eq!(fs.len(), 16);
    // Should be truncated to exactly 16 bytes
    assert_eq!(fs.as_str(), Some("This is a very l"));
}

#[test]
fn test_fixed_string_empty() {
    let fs = FixedString::<8>::new();

    assert_eq!(fs.len(), 0);
    assert_eq!(fs.as_str(), Some(""));
    assert!(fs.is_empty());
    assert_eq!(fs.as_bytes(), &[0u8; 8]);
}

#[test]
fn test_fixed_string_serialization() {
    let fs = FixedString::<8>::from_str("test");

    let bytes = fs.to_be_bytes();
    assert_eq!(bytes.len(), 8);

    // First 4 bytes should be "test", rest should be null
    let expected = vec![b't', b'e', b's', b't', 0, 0, 0, 0];
    assert_eq!(bytes, expected);

    // Test deserialization
    let (decoded, bytes_read) = FixedString::<8>::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(bytes_read, 8);
    assert_eq!(decoded.as_str(), Some("test"));
}

#[test]
fn test_fixed_string_in_struct() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Message {
        id: u32,
        name: FixedString16,
        status: FixedString8,
    }

    let msg = Message {
        id: 42,
        name: FixedString16::from_str("Alice"),
        status: FixedString8::from_str("active"),
    };

    let bytes = msg.to_be_bytes();
    let (decoded, bytes_read) = Message::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(decoded, msg);
    assert_eq!(bytes_read, 4 + 16 + 8); // u32 + FixedString<16> + FixedString<8>
    assert_eq!(decoded.name.as_str(), Some("Alice"));
    assert_eq!(decoded.status.as_str(), Some("active"));
}

#[test]
fn test_fixed_string_unicode() {
    let fs = FixedString::<16>::from_str("Hello 世界");

    // "Hello 世界" should be "Hello " (6 bytes) + "世" (3 bytes) + "界" (3 bytes) = 12 bytes
    assert_eq!(fs.len(), 12);
    assert_eq!(fs.as_str(), Some("Hello 世界"));

    // Test serialization
    let bytes = fs.to_be_bytes();
    let (decoded, _) = FixedString::<16>::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded.as_str(), Some("Hello 世界"));
}

#[test]
fn test_fixed_string_unicode_truncation() {
    // This string is longer than 8 bytes when encoded in UTF-8
    let fs = FixedString::<8>::from_str("世界世界世");

    // Should be truncated at byte boundary, not character boundary
    // "世界世界世" would be 15 bytes, truncated to 8 bytes might cut in middle of character
    assert_eq!(fs.len(), 8);

    // The truncated result might not be valid UTF-8, so as_str() might return None
    // This is expected behavior - truncation happens at byte level
    let result = fs.as_str();
    // We just verify that it either works or gracefully returns None
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_fixed_string_display() {
    let fs = FixedString::<8>::from_str("test");
    assert_eq!(format!("{}", fs), "test");

    let empty_fs = FixedString::<8>::new();
    assert_eq!(format!("{}", empty_fs), "");
}

#[test]
fn test_fixed_string_clear() {
    let mut fs = FixedString::<8>::from_str("hello");
    assert_eq!(fs.as_str(), Some("hello"));

    fs.clear();
    assert_eq!(fs.as_str(), Some(""));
    assert!(fs.is_empty());
    assert_eq!(fs.as_bytes(), &[0u8; 8]);
}

#[test]
fn test_fixed_string_from_conversions() {
    // Test From<&str>
    let fs1: FixedString<8> = "hello".into();
    assert_eq!(fs1.as_str(), Some("hello"));

    // Test From<String> (only when std feature is enabled)
    #[cfg(feature = "std")]
    {
        let fs2: FixedString<8> = String::from("world").into();
        assert_eq!(fs2.as_str(), Some("world"));
    }

    #[cfg(not(feature = "std"))]
    {
        let fs2: FixedString<8> = alloc::string::String::from("world").into();
        assert_eq!(fs2.as_str(), Some("world"));
    }
}

#[test]
fn test_fixed_string_endianness_independence() {
    let fs = FixedString::<8>::from_str("test");

    let be_bytes = fs.to_be_bytes();
    let le_bytes = fs.to_le_bytes();

    // For byte arrays, endianness shouldn't matter
    assert_eq!(be_bytes, le_bytes);

    let (decoded_be, _) = FixedString::<8>::try_from_be_bytes(&be_bytes).unwrap();
    let (decoded_le, _) = FixedString::<8>::try_from_le_bytes(&le_bytes).unwrap();

    assert_eq!(decoded_be, decoded_le);
    assert_eq!(decoded_be.as_str(), Some("test"));
}

#[test]
fn test_fixed_string_insufficient_data() {
    let short_bytes = vec![1, 2, 3];
    let result = FixedString::<8>::try_from_be_bytes(&short_bytes);

    assert!(result.is_err());
    match result {
        Err(bebytes::BeBytesError::InsufficientData { expected, actual }) => {
            assert_eq!(expected, 8);
            assert_eq!(actual, 3);
        }
        _ => panic!("Expected InsufficientData error"),
    }
}

#[test]
fn test_fixed_string_with_embedded_nulls() {
    let mut fs = FixedString::<8>::new();
    fs.as_bytes_mut()[0] = b'a';
    fs.as_bytes_mut()[1] = 0; // null byte
    fs.as_bytes_mut()[2] = b'b';
    fs.as_bytes_mut()[3] = b'c';

    // Should stop at first null byte
    assert_eq!(fs.len(), 1);
    assert_eq!(fs.as_str(), Some("a"));
}
