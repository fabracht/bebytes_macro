use bebytes::{BeBytes, CString};
#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[test]
fn test_cstring_basic_functionality() {
    let cs = CString::from_str("Hello");

    assert_eq!(cs.len(), 5);
    assert_eq!(cs.as_str(), Some("Hello"));
    assert!(!cs.is_empty());
    assert_eq!(cs.as_bytes(), b"Hello");
}

#[test]
fn test_cstring_serialization() {
    let cs = CString::from_str("test");

    let bytes = cs.to_be_bytes();
    // Should be: [b't', b'e', b's', b't', 0] = 5 bytes total
    assert_eq!(bytes, vec![b't', b'e', b's', b't', 0]);

    let (decoded, bytes_read) = CString::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(bytes_read, 5);
    assert_eq!(decoded.as_str(), Some("test"));
    assert_eq!(decoded, cs);
}

#[test]
fn test_cstring_empty() {
    let cs = CString::new();

    assert_eq!(cs.len(), 0);
    assert_eq!(cs.as_str(), Some(""));
    assert!(cs.is_empty());

    let bytes = cs.to_be_bytes();
    assert_eq!(bytes, vec![0]); // Just the null terminator

    let (decoded, bytes_read) = CString::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(bytes_read, 1);
    assert_eq!(decoded, cs);
}

#[test]
fn test_cstring_with_embedded_null() {
    // CString should truncate at the first null byte
    let cs = CString::from_str("hello\0world");

    assert_eq!(cs.len(), 5);
    assert_eq!(cs.as_str(), Some("hello"));

    let bytes = cs.to_be_bytes();
    assert_eq!(bytes, vec![b'h', b'e', b'l', b'l', b'o', 0]);
}

#[test]
fn test_cstring_in_struct() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Message {
        id: u32,
        name: CString,
        path: CString,
    }

    let msg = Message {
        id: 42,
        name: CString::from_str("alice"),
        path: CString::from_str("/home/alice"),
    };

    let bytes = msg.to_be_bytes();
    let (decoded, _) = Message::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(decoded, msg);
    assert_eq!(decoded.name.as_str(), Some("alice"));
    assert_eq!(decoded.path.as_str(), Some("/home/alice"));
}

#[test]
fn test_cstring_unicode() {
    let cs = CString::from_str("Hello 世界! 🦀");

    // "Hello 世界! 🦀" = "Hello " (6) + "世" (3) + "界" (3) + "! " (2) + "🦀" (4) = 18 bytes
    assert_eq!(cs.len(), 18);
    assert_eq!(cs.as_str(), Some("Hello 世界! 🦀"));

    let bytes = cs.to_be_bytes();
    assert_eq!(bytes.len(), 19); // 18 bytes content + 1 null terminator
    assert_eq!(bytes[18], 0); // Last byte is null terminator

    let (decoded, _) = CString::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded.as_str(), Some("Hello 世界! 🦀"));
}

#[test]
fn test_cstring_endianness_independence() {
    let cs = CString::from_str("test");

    let be_bytes = cs.to_be_bytes();
    let le_bytes = cs.to_le_bytes();

    // For byte data, endianness shouldn't matter
    assert_eq!(be_bytes, le_bytes);

    let (decoded_be, _) = CString::try_from_be_bytes(&be_bytes).unwrap();
    let (decoded_le, _) = CString::try_from_le_bytes(&le_bytes).unwrap();

    assert_eq!(decoded_be, decoded_le);
    assert_eq!(decoded_be.as_str(), Some("test"));
}

#[test]
fn test_cstring_missing_null_terminator() {
    // Test with bytes that don't have a null terminator
    let bytes = vec![b'h', b'e', b'l', b'l', b'o'];
    let result = CString::try_from_be_bytes(&bytes);

    assert!(result.is_err());
    match result {
        Err(bebytes::BeBytesError::InvalidDiscriminant { type_name, .. }) => {
            assert_eq!(type_name, "CString (missing null terminator)");
        }
        _ => panic!("Expected InvalidDiscriminant error for missing null terminator"),
    }
}

#[test]
fn test_cstring_invalid_utf8() {
    // Create bytes with invalid UTF-8 sequence followed by null terminator
    let bytes = vec![0xFF, 0xFE, 0xFD, 0]; // Invalid UTF-8 + null terminator

    let result = CString::try_from_be_bytes(&bytes);

    assert!(result.is_err());
    match result {
        Err(bebytes::BeBytesError::InvalidDiscriminant { type_name, .. }) => {
            assert_eq!(type_name, "CString (invalid UTF-8)");
        }
        _ => panic!("Expected InvalidDiscriminant error for UTF-8"),
    }
}

#[test]
fn test_cstring_manipulation() {
    let mut cs = CString::from_str("Hello");

    cs.push_str(", world");
    assert_eq!(cs.as_str(), Some("Hello, world"));
    assert_eq!(cs.len(), 12);

    cs.push('!');
    assert_eq!(cs.as_str(), Some("Hello, world!"));
    assert_eq!(cs.len(), 13);

    // Test that null characters are ignored in push operations
    cs.push('\0');
    assert_eq!(cs.as_str(), Some("Hello, world!"));
    assert_eq!(cs.len(), 13);

    cs.clear();
    assert_eq!(cs.as_str(), Some(""));
    assert_eq!(cs.len(), 0);
    assert!(cs.is_empty());
}

#[test]
fn test_cstring_from_conversions() {
    // Test From<&str>
    let cs1: CString = "hello".into();
    assert_eq!(cs1.as_str(), Some("hello"));

    // Test From<String>
    #[cfg(feature = "std")]
    {
        let cs2: CString = String::from("world").into();
        assert_eq!(cs2.as_str(), Some("world"));
    }

    #[cfg(not(feature = "std"))]
    {
        let cs2: CString = alloc::string::String::from("world").into();
        assert_eq!(cs2.as_str(), Some("world"));
    }
}

#[test]
fn test_cstring_display() {
    let cs = CString::from_str("test");
    assert_eq!(format!("{}", cs), "test");

    let empty_cs = CString::new();
    assert_eq!(format!("{}", empty_cs), "");
}

#[test]
fn test_cstring_complex_struct() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct FileEntry {
        size: u64,
        filename: CString,
        path: CString,
        owner: CString,
        permissions: u32,
    }

    let entry = FileEntry {
        size: 1024,
        filename: CString::from_str("document.txt"),
        path: CString::from_str("/home/user/documents/document.txt"),
        owner: CString::from_str("user"),
        permissions: 0o644,
    };

    let bytes = entry.to_be_bytes();
    let (decoded, _) = FileEntry::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(decoded, entry);
    assert_eq!(decoded.filename.as_str(), Some("document.txt"));
    assert_eq!(
        decoded.path.as_str(),
        Some("/home/user/documents/document.txt")
    );
    assert_eq!(decoded.owner.as_str(), Some("user"));
}

#[test]
fn test_cstring_multiple_in_sequence() {
    // Test multiple CStrings in sequence to ensure proper parsing
    let cs1 = CString::from_str("first");
    let cs2 = CString::from_str("second");
    let cs3 = CString::from_str("third");

    let mut combined = Vec::new();
    combined.extend_from_slice(&cs1.to_be_bytes());
    combined.extend_from_slice(&cs2.to_be_bytes());
    combined.extend_from_slice(&cs3.to_be_bytes());

    // Should be: "first\0second\0third\0"
    let expected = b"first\0second\0third\0".to_vec();
    assert_eq!(combined, expected);

    // Parse them back
    let mut offset = 0;
    let (parsed1, consumed1) = CString::try_from_be_bytes(&combined[offset..]).unwrap();
    offset += consumed1;
    let (parsed2, consumed2) = CString::try_from_be_bytes(&combined[offset..]).unwrap();
    offset += consumed2;
    let (parsed3, consumed3) = CString::try_from_be_bytes(&combined[offset..]).unwrap();
    offset += consumed3;

    assert_eq!(parsed1, cs1);
    assert_eq!(parsed2, cs2);
    assert_eq!(parsed3, cs3);
    assert_eq!(offset, combined.len());
}

#[test]
fn test_cstring_push_str_with_null() {
    let mut cs = CString::from_str("hello");

    // push_str should stop at null byte
    cs.push_str(" world\0ignored");
    assert_eq!(cs.as_str(), Some("hello world"));
    assert_eq!(cs.len(), 11);
}
