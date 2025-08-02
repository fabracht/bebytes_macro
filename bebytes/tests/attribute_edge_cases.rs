//! Tests for attribute parsing edge cases
//! These target mutations in parse_attributes that return wrong values

use bebytes::BeBytes;

#[test]
fn test_bits_attribute_values() {
    // Test various bit sizes to ensure they're parsed correctly
    // Not as Some(0), Some(1), or None

    #[derive(BeBytes, Debug, PartialEq)]
    struct BitsTest2 {
        #[bits(2)]
        two_bits: u8,
        #[bits(6)]
        six_bits: u8,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct BitsTest10 {
        #[bits(10)]
        ten_bits: u16,
        #[bits(6)]
        six_bits: u8,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct BitsTest31 {
        #[bits(31)]
        thirty_one_bits: u32,
        #[bits(1)]
        one_bit: u8,
    }

    // Test that different bit sizes work correctly
    assert_eq!(BitsTest2::field_size(), 1); // 2 + 6 = 8 bits
    assert_eq!(BitsTest10::field_size(), 2); // 10 + 6 = 16 bits
    assert_eq!(BitsTest31::field_size(), 4); // 31 + 1 = 32 bits

    // Test actual values
    let test2 = BitsTest2 {
        two_bits: 3,  // max 2-bit value
        six_bits: 63, // max 6-bit value
    };
    let bytes = test2.to_be_bytes();
    assert_eq!(bytes[0], 0xFF); // 11_111111

    let test10 = BitsTest10 {
        ten_bits: 1023, // max 10-bit value
        six_bits: 63,   // max 6-bit value
    };
    let bytes = test10.to_be_bytes();
    assert_eq!(bytes.len(), 2);

    let test31 = BitsTest31 {
        thirty_one_bits: 0x7FFFFFFF, // max 31-bit value
        one_bit: 1,
    };
    let bytes = test31.to_be_bytes();
    assert_eq!(bytes.len(), 4);
}

#[test]
fn test_no_attributes() {
    // Test struct with no bit attributes
    #[derive(BeBytes, Debug, PartialEq)]
    struct NoAttributes {
        a: u8,
        b: u16,
        c: u32,
    }

    assert_eq!(NoAttributes::field_size(), 7);

    let test = NoAttributes {
        a: 0xFF,
        b: 0xFFFF,
        c: 0xFFFFFFFF,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 7);

    let (parsed, _) = NoAttributes::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, test);
}

#[test]
fn test_mixed_attributes() {
    // Test mix of fields with and without attributes
    #[derive(BeBytes, Debug, PartialEq)]
    struct MixedAttributes {
        #[bits(4)]
        nibble: u8,
        regular: u8,
        #[bits(12)]
        twelve: u16,
    }

    assert_eq!(MixedAttributes::field_size(), 3); // 4 + 8 + 12 = 24 bits

    let test = MixedAttributes {
        nibble: 15,
        regular: 255,
        twelve: 4095,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 3);
}

#[test]
fn test_from_field_attribute() {
    // Test FromField attribute parsing
    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct Header {
        count: u16,
        flags: u8,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct FromFieldTest {
        header: Header,
        #[FromField(header.count)]
        data: Vec<u8>,
    }

    let test = FromFieldTest {
        header: Header {
            count: 3,
            flags: 0xFF,
        },
        data: vec![1, 2, 3],
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 6); // 2 + 1 + 3

    // Parse and verify
    let (parsed, _) = FromFieldTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.header.count, 3);
    assert_eq!(parsed.header.flags, 0xFF);
    assert_eq!(parsed.data, vec![1, 2, 3]);
}

#[test]
fn test_fixed_size_arrays() {
    // Test fixed size arrays instead of With attribute
    #[derive(BeBytes, Debug, PartialEq)]
    struct FixedArrayTest {
        prefix: u8,
        fixed_data: [u8; 5],
        suffix: u8,
    }

    let test = FixedArrayTest {
        prefix: 0xAA,
        fixed_data: [1, 2, 3, 4, 5],
        suffix: 0xBB,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 7); // 1 + 5 + 1
    assert_eq!(bytes[0], 0xAA);
    assert_eq!(&bytes[1..6], &[1, 2, 3, 4, 5]);
    assert_eq!(bytes[6], 0xBB);
}

#[test]
fn test_zero_bit_field() {
    // Edge case: what if someone tries bits(0)?
    // This should fail at compile time, but let's test bits(8) which is like no attribute
    #[derive(BeBytes, Debug, PartialEq)]
    struct ZeroBitTest {
        #[bits(8)]
        full_byte: u8,
        #[bits(16)]
        two_bytes: u16,
    }

    assert_eq!(ZeroBitTest::field_size(), 3);

    let test = ZeroBitTest {
        full_byte: 255,
        two_bytes: 65535,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 3);
    assert_eq!(bytes[0], 255);
    assert_eq!(bytes[1], 255);
    assert_eq!(bytes[2], 255);
}

#[test]
fn test_consecutive_bit_fields() {
    // Test many consecutive bit fields
    #[derive(BeBytes, Debug, PartialEq)]
    struct ConsecutiveBits {
        #[bits(1)]
        bit1: u8,
        #[bits(2)]
        bit2: u8,
        #[bits(3)]
        bit3: u8,
        #[bits(4)]
        bit4: u8,
        #[bits(5)]
        bit5: u8,
        #[bits(6)]
        bit6: u8,
        #[bits(7)]
        bit7: u8,
        #[bits(4)]
        padding: u8, // Add 4 bits to make 32 bits total
    }

    // Total: 1+2+3+4+5+6+7+4 = 32 bits = 4 bytes
    assert_eq!(ConsecutiveBits::field_size(), 4);

    let test = ConsecutiveBits {
        bit1: 1,
        bit2: 3,
        bit3: 7,
        bit4: 15,
        bit5: 31,
        bit6: 63,
        bit7: 127,
        padding: 0,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 4);

    // Parse back
    let (parsed, _) = ConsecutiveBits::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.bit1, 1);
    assert_eq!(parsed.bit2, 3);
    assert_eq!(parsed.bit3, 7);
    assert_eq!(parsed.bit4, 15);
    assert_eq!(parsed.bit5, 31);
    assert_eq!(parsed.bit6, 63);
    assert_eq!(parsed.bit7, 127);
}

#[test]
fn test_enum_with_attributes() {
    // Test enums work correctly with bit fields
    #[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
    enum TestEnum {
        A = 0,
        B = 1,
        C = 2,
        D = 3,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct EnumWithBits {
        #[bits(2)]
        two_bit_enum: u8, // Can hold values 0-3
        #[bits(6)]
        padding: u8,
        full_enum: TestEnum,
    }

    let test = EnumWithBits {
        two_bit_enum: 3,
        padding: 0,
        full_enum: TestEnum::D,
    };

    assert_eq!(EnumWithBits::field_size(), 2); // 8 + 8 = 16 bits

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 2);

    let (parsed, _) = EnumWithBits::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.two_bit_enum, 3);
    assert_eq!(parsed.full_enum as u8, 3);
}
