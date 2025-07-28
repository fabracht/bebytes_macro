use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[test]
fn test_char_primitive_big_endian() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct CharStruct {
        ch: char,
    }

    let test_struct = CharStruct { ch: 'A' };
    let bytes = test_struct.to_be_bytes();
    
    // 'A' is Unicode U+0041, which should be [0, 0, 0, 65] in big endian
    assert_eq!(bytes, vec![0, 0, 0, 65]);
    
    let (decoded, bytes_read) = CharStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded, test_struct);
    assert_eq!(bytes_read, 4);
}

#[test]
fn test_char_primitive_little_endian() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct CharStruct {
        ch: char,
    }

    let test_struct = CharStruct { ch: 'A' };
    let bytes = test_struct.to_le_bytes();
    
    // 'A' is Unicode U+0041, which should be [65, 0, 0, 0] in little endian
    assert_eq!(bytes, vec![65, 0, 0, 0]);
    
    let (decoded, bytes_read) = CharStruct::try_from_le_bytes(&bytes).unwrap();
    assert_eq!(decoded, test_struct);
    assert_eq!(bytes_read, 4);
}

#[test]
fn test_char_unicode_characters() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct UnicodeStruct {
        emoji: char,
        chinese: char,
        ascii: char,
    }

    let test_struct = UnicodeStruct {
        emoji: '🦀',     // U+1F980
        chinese: '中',   // U+4E2D
        ascii: 'Z',      // U+005A
    };
    
    let bytes = test_struct.to_be_bytes();
    let (decoded, bytes_read) = UnicodeStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded, test_struct);
    assert_eq!(bytes_read, 12); // 3 chars * 4 bytes each
}

#[test]
fn test_char_bit_fields_aligned() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct CharBitStruct {
        #[bits(32)]
        ch: char,
    }

    let test_struct = CharBitStruct { ch: 'B' };
    let bytes = test_struct.to_be_bytes();
    
    // 'B' is Unicode U+0042
    assert_eq!(bytes, vec![0, 0, 0, 66]);
    
    let (decoded, bytes_read) = CharBitStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded, test_struct);
    assert_eq!(bytes_read, 4);
}

#[test]
fn test_char_bit_fields_with_other_types() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct MixedBitStruct {
        #[bits(8)]
        flags: u8,
        #[bits(16)]
        ch: char,
        #[bits(8)]
        more_flags: u8,
    }

    let test_struct = MixedBitStruct {
        flags: 255,        // 0xFF
        ch: 'C',           // U+0043 = 67 (fits in 16 bits)
        more_flags: 170,   // 0xAA
    };
    
    let bytes = test_struct.to_be_bytes();
    let (decoded, bytes_read) = MixedBitStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded, test_struct);
    assert_eq!(bytes_read, 4); // 8 + 16 + 8 = 32 bits = 4 bytes
}

#[test]
fn test_char_bit_fields_unaligned() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct UnalignedCharStruct {
        #[bits(1)]
        flag: u8,
        #[bits(15)]
        ch: char,
    }

    let test_struct = UnalignedCharStruct {
        flag: 1,
        ch: 'D', // U+0044 = 68
    };
    
    let bytes = test_struct.to_be_bytes();
    let (decoded, bytes_read) = UnalignedCharStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded, test_struct);
    assert_eq!(bytes_read, 2); // 1 + 15 = 16 bits = 2 bytes
}

#[test]
fn test_char_with_high_unicode_values() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct HighUnicodeStruct {
        #[bits(21)]
        ch: char, // Test with 21 bits to handle high Unicode values
        #[bits(3)]
        padding: u8,
    }

    let test_struct = HighUnicodeStruct {
        ch: '🎯', // U+1F3AF (high Unicode value)
        padding: 7,
    };
    
    let bytes = test_struct.to_be_bytes();
    let (decoded, bytes_read) = HighUnicodeStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded, test_struct);
    assert_eq!(bytes_read, 3); // 21 + 3 = 24 bits = 3 bytes
}

#[test]
fn test_char_validation_failure() {
    // Test that invalid Unicode values are rejected
    #[derive(BeBytes, Debug, PartialEq)]
    struct CharStruct {
        ch: char,
    }

    // Create bytes with an invalid Unicode scalar value
    let invalid_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF]; // U+FFFFFFFF is not a valid char
    
    let result = CharStruct::try_from_be_bytes(&invalid_bytes);
    assert!(result.is_err(), "Should reject invalid Unicode values");
}

#[test]
fn test_char_bit_field_validation_failure() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct CharBitStruct {
        #[bits(32)]
        ch: char,
    }

    // Create bytes with an invalid Unicode scalar value in bit field
    let invalid_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF];
    
    let result = CharBitStruct::try_from_be_bytes(&invalid_bytes);
    assert!(result.is_err(), "Should reject invalid Unicode values in bit fields");
}

#[test]
fn test_char_surrogate_rejection() {
    // Test that surrogate code points are rejected (U+D800-U+DFFF)
    #[derive(BeBytes, Debug, PartialEq)]
    struct CharStruct {
        ch: char,
    }

    // U+D800 is a high surrogate, not a valid Unicode scalar value
    let surrogate_bytes = vec![0x00, 0x00, 0xD8, 0x00];
    
    let result = CharStruct::try_from_be_bytes(&surrogate_bytes);
    assert!(result.is_err(), "Should reject surrogate code points");
}

#[test]
fn test_char_edge_cases() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct EdgeCaseStruct {
        null_char: char,
        max_ascii: char,
        max_bmp: char,
    }

    let test_struct = EdgeCaseStruct {
        null_char: '\0',    // U+0000
        max_ascii: '\x7F',  // U+007F (max ASCII)
        max_bmp: '\u{FFFD}', // U+FFFD (replacement character, near max BMP)
    };
    
    let bytes = test_struct.to_be_bytes();
    let (decoded, bytes_read) = EdgeCaseStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded, test_struct);
    assert_eq!(bytes_read, 12); // 3 chars * 4 bytes each
}

#[test]
fn test_char_mixed_with_other_primitives() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct MixedStruct {
        byte_val: u8,
        char_val: char,
        int_val: u32,
        another_char: char,
    }

    let test_struct = MixedStruct {
        byte_val: 42,
        char_val: 'X',
        int_val: 0x12345678,
        another_char: '世', // Chinese character
    };
    
    let bytes = test_struct.to_be_bytes();
    let (decoded, bytes_read) = MixedStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded, test_struct);
    assert_eq!(bytes_read, 13); // 1 + 4 + 4 + 4 = 13 bytes
}