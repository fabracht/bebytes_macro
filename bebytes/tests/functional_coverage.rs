//! Tests targeting functional.rs mutations
//! These ensure parsing and writing functions work correctly

use bebytes::BeBytes;

#[test]
fn test_primitive_parsing_all_sizes() {
    // Test parsing for each primitive size - mutations that delete match arms will fail

    // 1-byte parsing
    #[derive(BeBytes, Debug, PartialEq)]
    struct OneByte {
        val: u8,
    }
    let bytes = vec![0x42];
    let (parsed, _) = OneByte::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.val, 0x42);

    // 2-byte parsing
    #[derive(BeBytes, Debug, PartialEq)]
    struct TwoByte {
        val: u16,
    }
    let bytes = vec![0x12, 0x34];
    let (parsed, _) = TwoByte::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.val, 0x1234);

    // 4-byte parsing
    #[derive(BeBytes, Debug, PartialEq)]
    struct FourByte {
        val: u32,
    }
    let bytes = vec![0x12, 0x34, 0x56, 0x78];
    let (parsed, _) = FourByte::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.val, 0x12345678);

    // 8-byte parsing
    #[derive(BeBytes, Debug, PartialEq)]
    struct EightByte {
        val: u64,
    }
    let bytes = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
    let (parsed, _) = EightByte::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.val, 0x123456789ABCDEF0);

    // 16-byte parsing
    #[derive(BeBytes, Debug, PartialEq)]
    struct SixteenByte {
        val: u128,
    }
    let bytes = vec![
        0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE,
        0xF0,
    ];
    let (parsed, _) = SixteenByte::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.val, 0x123456789ABCDEF0123456789ABCDEF0);
}

#[test]
fn test_primitive_writing_all_sizes() {
    // Test writing for each primitive size

    // 1-byte writing
    #[derive(BeBytes, Debug, PartialEq)]
    struct OneByte {
        val: u8,
    }
    let test = OneByte { val: 0x42 };
    let bytes = test.to_be_bytes();
    assert_eq!(bytes, vec![0x42]);

    // 2-byte writing
    #[derive(BeBytes, Debug, PartialEq)]
    struct TwoByte {
        val: u16,
    }
    let test = TwoByte { val: 0x1234 };
    let bytes = test.to_be_bytes();
    assert_eq!(bytes, vec![0x12, 0x34]);

    // 4-byte writing
    #[derive(BeBytes, Debug, PartialEq)]
    struct FourByte {
        val: u32,
    }
    let test = FourByte { val: 0x12345678 };
    let bytes = test.to_be_bytes();
    assert_eq!(bytes, vec![0x12, 0x34, 0x56, 0x78]);

    // 8-byte writing
    #[derive(BeBytes, Debug, PartialEq)]
    struct EightByte {
        val: u64,
    }
    let test = EightByte {
        val: 0x123456789ABCDEF0,
    };
    let bytes = test.to_be_bytes();
    assert_eq!(bytes, vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0]);
}

#[test]
fn test_bit_field_limit_check() {
    // Test that bit field limit checks work (mutations changing << to >> will fail)
    #[derive(BeBytes, Debug, PartialEq)]
    struct BitLimitTest {
        #[bits(4)]
        nibble: u8,
        #[bits(12)]
        twelve: u16,
    }

    // Values at the limit
    let test = BitLimitTest {
        nibble: 15,   // 2^4 - 1
        twelve: 4095, // 2^12 - 1
    };

    let bytes = test.to_be_bytes();
    let (parsed, _) = BitLimitTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.nibble, 15);
    assert_eq!(parsed.twelve, 4095);

    // Test edge case values
    let edge = BitLimitTest {
        nibble: 14,
        twelve: 4094,
    };

    let bytes_edge = edge.to_be_bytes();
    let (parsed_edge, _) = BitLimitTest::try_from_be_bytes(&bytes_edge).unwrap();
    assert_eq!(parsed_edge.nibble, 14);
    assert_eq!(parsed_edge.twelve, 4094);
}

#[test]
fn test_unaligned_multibyte_operations() {
    // Test unaligned multibyte parsing and writing
    #[derive(BeBytes, Debug, PartialEq)]
    struct UnalignedTest {
        #[bits(12)]
        twelve: u16,
        #[bits(20)]
        twenty: u32,
    }

    let test = UnalignedTest {
        twelve: 0xFFF,
        twenty: 0xFFFFF,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 4); // 32 bits = 4 bytes

    // Need to provide enough bytes for the underlying type
    let mut parse_bytes = vec![0u8; 10];
    parse_bytes[..bytes.len()].copy_from_slice(&bytes);
    let (parsed, _) = UnalignedTest::try_from_be_bytes(&parse_bytes).unwrap();
    assert_eq!(parsed.twelve, 0xFFF);
    assert_eq!(parsed.twenty, 0xFFFFF);
}

#[test]
fn test_single_byte_bit_operations() {
    // Test single byte bit parsing and writing
    #[derive(BeBytes, Debug, PartialEq)]
    struct SingleByteTest {
        #[bits(2)]
        two: u8,
        #[bits(3)]
        three: u8,
        #[bits(3)]
        three2: u8,
    }

    let test = SingleByteTest {
        two: 0b11,
        three: 0b111,
        three2: 0b101,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 1);

    // Expected: 11_111_101 = 0xFD
    assert_eq!(bytes[0], 0xFD);

    let (parsed, _) = SingleByteTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.two, 0b11);
    assert_eq!(parsed.three, 0b111);
    assert_eq!(parsed.three2, 0b101);
}

#[test]
fn test_field_access_path() {
    // Test field access path generation
    #[derive(BeBytes, Debug, PartialEq)]
    struct NestedAccess {
        header: u16,
        #[FromField(header)]
        data: Vec<u8>,
    }

    let test = NestedAccess {
        header: 3,
        data: vec![1, 2, 3],
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 5); // 2 + 3
    assert_eq!(&bytes[0..2], &[0, 3]); // header
    assert_eq!(&bytes[2..5], &[1, 2, 3]); // data
}

#[test]
fn test_byte_completeness_validation() {
    // This would fail at compile time if validation is broken
    // The fact it compiles means bit validation works
    #[derive(BeBytes, Debug, PartialEq)]
    struct ValidBits {
        #[bits(8)]
        one_byte: u8,
        #[bits(16)]
        two_bytes: u16,
        #[bits(24)]
        three_bytes: u32,
    }

    assert_eq!(ValidBits::field_size(), 6); // 48 bits = 6 bytes
}

#[test]
fn test_endianness_consistency() {
    // Test that BE and LE use correct methods
    #[derive(BeBytes, Debug, PartialEq)]
    struct EndianTest {
        value: u32,
    }

    let test = EndianTest { value: 0x12345678 };

    let be_bytes = test.to_be_bytes();
    let le_bytes = test.to_le_bytes();

    // BE should be big-endian
    assert_eq!(be_bytes, vec![0x12, 0x34, 0x56, 0x78]);
    // LE should be little-endian
    assert_eq!(le_bytes, vec![0x78, 0x56, 0x34, 0x12]);

    // Parse BE
    let (be_parsed, _) = EndianTest::try_from_be_bytes(&be_bytes).unwrap();
    assert_eq!(be_parsed.value, 0x12345678);

    // Parse LE
    let (le_parsed, _) = EndianTest::try_from_le_bytes(&le_bytes).unwrap();
    assert_eq!(le_parsed.value, 0x12345678);
}

#[test]
fn test_attribute_merge_operations() {
    // Test that |= operations work correctly (not ^=)
    #[derive(BeBytes, Debug, PartialEq)]
    struct MergeTest {
        #[bits(4)]
        first: u8,
        #[bits(4)]
        second: u8,
    }

    let test = MergeTest {
        first: 0xF,
        second: 0xF,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes[0], 0xFF);

    let (parsed, _) = MergeTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.first, 0xF);
    assert_eq!(parsed.second, 0xF);
}
