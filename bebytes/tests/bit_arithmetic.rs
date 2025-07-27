//! Tests that validate bit arithmetic operations work correctly
//! This addresses mutations where * 8 could be replaced with + 8 without tests failing

use bebytes::BeBytes;

#[test]
fn test_bit_to_byte_conversion() {
    // Test that bit fields correctly convert to bytes
    #[derive(BeBytes, Debug, PartialEq)]
    struct BitToByteTest {
        #[bits(8)]
        one_byte: u8,
        #[bits(16)]
        two_bytes: u16,
        #[bits(32)]
        four_bytes: u32,
    }

    let test = BitToByteTest {
        one_byte: 0xFF,
        two_bytes: 0xFFFF,
        four_bytes: 0xFFFFFFFF,
    };

    // 8 + 16 + 32 = 56 bits = 7 bytes
    assert_eq!(BitToByteTest::field_size(), 7);

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 7);

    // Verify the actual byte layout
    assert_eq!(bytes[0], 0xFF); // one_byte
    assert_eq!(bytes[1], 0xFF); // two_bytes high
    assert_eq!(bytes[2], 0xFF); // two_bytes low
    assert_eq!(bytes[3], 0xFF); // four_bytes highest
    assert_eq!(bytes[4], 0xFF); // four_bytes high
    assert_eq!(bytes[5], 0xFF); // four_bytes low
    assert_eq!(bytes[6], 0xFF); // four_bytes lowest
}

#[test]
fn test_unaligned_bit_fields() {
    // Test fields that don't align to byte boundaries
    #[derive(BeBytes, Debug, PartialEq)]
    struct UnalignedBits {
        #[bits(3)]
        three_bits: u8,
        #[bits(5)]
        five_bits: u8,
        #[bits(7)]
        seven_bits: u8,
        #[bits(9)]
        nine_bits: u16,
    }

    let test = UnalignedBits {
        three_bits: 0b111,
        five_bits: 0b11111,
        seven_bits: 0b1111111,
        nine_bits: 0b111111111,
    };

    // 3 + 5 + 7 + 9 = 24 bits = 3 bytes
    assert_eq!(UnalignedBits::field_size(), 3);

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 3);

    // Verify parsing works correctly
    let (parsed, consumed) = UnalignedBits::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, test);
    assert_eq!(consumed, 3);
}

#[test]
fn test_byte_alignment_padding() {
    // Test that partial bytes are properly padded
    #[derive(BeBytes, Debug, PartialEq)]
    struct PaddingTest {
        #[bits(4)]
        nibble: u8,
        #[bits(4)]
        another_nibble: u8,
        #[bits(3)]
        three_bits: u8,
        #[bits(5)]
        five_bits: u8,
    }

    let test = PaddingTest {
        nibble: 0xF,
        another_nibble: 0xF,
        three_bits: 0b111,
        five_bits: 0b11111,
    };

    // 4 + 4 + 3 + 5 = 16 bits = 2 bytes
    assert_eq!(PaddingTest::field_size(), 2);

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 2);

    // First byte: nibble (4 bits) + another_nibble (4 bits) = 0xFF
    assert_eq!(bytes[0], 0xFF);
    // Second byte: three_bits (3 bits) + five_bits (5 bits) = 0xFF
    assert_eq!(bytes[1], 0xFF);
}

#[test]
fn test_large_bit_fields() {
    // Test fields with explicit byte sizes (not bit attributes)
    #[derive(BeBytes, Debug, PartialEq)]
    struct LargeBitFields {
        large_field: u64,       // 8 bytes
        very_large_field: u128, // 16 bytes
    }

    let test = LargeBitFields {
        large_field: 0xFFFFFFFFFFFFFFFF,
        very_large_field: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    };

    // 8 + 16 = 24 bytes
    assert_eq!(LargeBitFields::field_size(), 24);

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 24);

    // All bytes should be 0xFF
    assert!(bytes.iter().all(|&b| b == 0xFF));
}

#[test]
fn test_div_ceil_behavior() {
    // Test edge cases that demonstrate bit to byte conversion
    // All structs must have complete bytes

    #[derive(BeBytes, Debug, PartialEq)]
    struct OneByteStruct {
        #[bits(1)]
        one_bit: u8,
        #[bits(7)]
        seven_bits: u8,
    }

    // 1 + 7 = 8 bits = 1 byte
    assert_eq!(OneByteStruct::field_size(), 1);

    #[derive(BeBytes, Debug, PartialEq)]
    struct TwoByteStruct {
        #[bits(9)]
        nine_bits: u16,
        #[bits(7)]
        seven_bits: u8,
    }

    // 9 + 7 = 16 bits = 2 bytes
    assert_eq!(TwoByteStruct::field_size(), 2);

    #[derive(BeBytes, Debug, PartialEq)]
    struct ThreeByteStruct {
        #[bits(17)]
        seventeen_bits: u32,
        #[bits(7)]
        seven_bits: u8,
    }

    // 17 + 7 = 24 bits = 3 bytes
    assert_eq!(ThreeByteStruct::field_size(), 3);

    // Test values that would overflow if using + instead of *
    let test = ThreeByteStruct {
        seventeen_bits: 0x1FFFF, // Max value for 17 bits
        seven_bits: 0x7F,        // Max value for 7 bits
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 3);

    // Let's verify the byte representation
    // 17 bits: 0x1FFFF = 0b1_1111_1111_1111_1111
    // 7 bits:  0x7F     = 0b111_1111
    // Combined: 0b111_1111_1111_1111_1111_1111_111 (24 bits = 3 bytes)
    assert_eq!(bytes[0], 0xFF); // First 8 bits of seventeen_bits
    assert_eq!(bytes[1], 0xFF); // Next 8 bits of seventeen_bits
    assert_eq!(bytes[2], 0xFF); // Last 1 bit of seventeen_bits + 7 bits of seven_bits

    // Verify parsing preserves values
    // Need to provide enough bytes for u32 even though we only use 3
    let mut parse_bytes = vec![0u8; 10];
    parse_bytes[..3].copy_from_slice(&bytes);
    let (parsed, consumed) = ThreeByteStruct::try_from_be_bytes(&parse_bytes).unwrap();
    assert_eq!(consumed, 3);
    assert_eq!(parsed.seventeen_bits, 0x1FFFF);
    assert_eq!(parsed.seven_bits, 0x7F);
}

#[test]
fn test_mixed_aligned_unaligned() {
    // Test mix of byte-aligned and unaligned fields
    #[derive(BeBytes, Debug, PartialEq)]
    struct MixedAlignment {
        regular_u8: u8, // 8 bits (aligned)
        #[bits(3)]
        three_bits: u8, // 3 bits (unaligned)
        #[bits(5)]
        five_bits: u8, // 5 bits (completes byte)
        regular_u16: u16, // 16 bits (aligned)
        #[bits(12)]
        twelve_bits: u16, // 12 bits (unaligned)
        #[bits(4)]
        four_bits: u8, // 4 bits (completes byte)
    }

    // 8 + 3 + 5 + 16 + 12 + 4 = 48 bits = 6 bytes
    assert_eq!(MixedAlignment::field_size(), 6);

    let test = MixedAlignment {
        regular_u8: 0xFF,
        three_bits: 0b111,
        five_bits: 0b11111,
        regular_u16: 0xABCD,
        twelve_bits: 0xFFF,
        four_bits: 0xF,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 6);

    // Parse back and verify
    let (parsed, consumed) = MixedAlignment::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, test);
    assert_eq!(consumed, 6);
}

#[test]
fn test_zero_values_bit_arithmetic() {
    // Test that arithmetic works with zero values
    #[derive(BeBytes, Debug, PartialEq)]
    struct ZeroTest {
        #[bits(3)]
        three_bits: u8,
        #[bits(5)]
        five_bits: u8,
        regular: u32,
    }

    let test = ZeroTest {
        three_bits: 0,
        five_bits: 0,
        regular: 0,
    };

    // 3 + 5 + 32 = 40 bits = 5 bytes
    assert_eq!(ZeroTest::field_size(), 5);

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 5);
    assert_eq!(bytes, vec![0, 0, 0, 0, 0]);
}

#[test]
fn test_single_bit_fields() {
    // Test edge case of single bit fields
    #[derive(BeBytes, Debug, PartialEq)]
    struct SingleBitFlags {
        #[bits(1)]
        flag1: u8,
        #[bits(1)]
        flag2: u8,
        #[bits(1)]
        flag3: u8,
        #[bits(1)]
        flag4: u8,
        #[bits(1)]
        flag5: u8,
        #[bits(1)]
        flag6: u8,
        #[bits(1)]
        flag7: u8,
        #[bits(1)]
        flag8: u8,
    }

    // 8 * 1 = 8 bits = 1 byte
    assert_eq!(SingleBitFlags::field_size(), 1);

    let test = SingleBitFlags {
        flag1: 1,
        flag2: 0,
        flag3: 1,
        flag4: 0,
        flag5: 1,
        flag6: 0,
        flag7: 1,
        flag8: 0,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 1);
    assert_eq!(bytes[0], 0b10101010);
}

#[test]
fn test_array_field_bit_calculation() {
    // Test that array fields calculate bits correctly
    #[derive(BeBytes, Debug, PartialEq)]
    struct ArrayTest {
        #[bits(4)]
        nibble: u8,
        #[bits(4)]
        another: u8,
        data: [u8; 10], // 10 * 8 = 80 bits
    }

    // 4 + 4 + 80 = 88 bits = 11 bytes
    assert_eq!(ArrayTest::field_size(), 11);

    let test = ArrayTest {
        nibble: 0xF,
        another: 0xF,
        data: [0xAA; 10],
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 11);

    // Verify the data starts at byte 1
    assert_eq!(bytes[0], 0xFF); // nibble + another
    for i in 1..11 {
        assert_eq!(bytes[i], 0xAA);
    }
}

#[test]
fn test_enum_bit_field_sizes() {
    // Test that enums work with regular byte alignment
    #[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
    #[repr(u8)]
    enum TestEnum {
        A = 0,
        B = 1,
        C = 2,
        D = 3,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct EnumTest {
        first_enum: TestEnum, // 1 byte
        #[bits(2)]
        two_bits: u8,
        #[bits(6)]
        six_bits: u8,
        second_enum: TestEnum, // 1 byte
        #[bits(3)]
        three_bits: u8,
        #[bits(5)]
        five_bits: u8,
    }

    // 1 + (2 + 6) + 1 + (3 + 5) = 1 + 1 + 1 + 1 = 4 bytes
    assert_eq!(EnumTest::field_size(), 4);

    let test = EnumTest {
        first_enum: TestEnum::D,
        two_bits: 0b11,
        six_bits: 0b111111,
        second_enum: TestEnum::C,
        three_bits: 0b111,
        five_bits: 0b11111,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 4);

    let (parsed, _) = EnumTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, test);
}
