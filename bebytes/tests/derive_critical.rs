//! Critical tests to ensure derive_be_bytes actually generates working code
//! These tests specifically target mutations that would break core functionality

use bebytes::BeBytes;

#[test]
fn test_derive_generates_working_code() {
    // This test ensures derive_be_bytes doesn't just return Default::default()
    #[derive(BeBytes)]
    struct Critical {
        value: u32,
    }

    let test = Critical { value: 0x12345678 };

    // If derive returned Default::default(), these would panic
    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 4);
    assert_eq!(bytes, vec![0x12, 0x34, 0x56, 0x78]);

    let (parsed, consumed) = Critical::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, 4);
    assert_eq!(parsed.value, 0x12345678);

    // Test that field_size is correct
    assert_eq!(Critical::field_size(), 4);
}

#[test]
fn test_all_primitive_sizes_work() {
    // Test u8
    #[derive(BeBytes, Debug, PartialEq)]
    struct U8Test {
        value: u8,
    }
    assert_eq!(U8Test::field_size(), 1);
    let test = U8Test { value: 0xFF };
    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 1);

    // Test u16
    #[derive(BeBytes, Debug, PartialEq)]
    struct U16Test {
        value: u16,
    }
    assert_eq!(U16Test::field_size(), 2);
    let test = U16Test { value: 0xFFFF };
    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 2);

    // Test u32
    #[derive(BeBytes, Debug, PartialEq)]
    struct U32Test {
        value: u32,
    }
    assert_eq!(U32Test::field_size(), 4);
    let test = U32Test { value: 0xFFFFFFFF };
    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 4);

    // Test u64
    #[derive(BeBytes, Debug, PartialEq)]
    struct U64Test {
        value: u64,
    }
    assert_eq!(U64Test::field_size(), 8);
    let test = U64Test {
        value: 0xFFFFFFFFFFFFFFFF,
    };
    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 8);

    // Test u128
    #[derive(BeBytes, Debug, PartialEq)]
    struct U128Test {
        value: u128,
    }
    assert_eq!(U128Test::field_size(), 16);
    let test = U128Test {
        value: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    };
    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 16);
}

#[test]
fn test_bit_arithmetic_multiplication() {
    // Test that bit to byte conversion uses multiplication, not division
    #[derive(BeBytes, Debug, PartialEq)]
    struct BitMultiplyTest {
        array: [u8; 10], // 10 * 8 = 80 bits
    }

    // If * is replaced with /, this would give 10 / 8 = 1 byte instead of 80 bits
    assert_eq!(BitMultiplyTest::field_size(), 10);

    let test = BitMultiplyTest { array: [0xAA; 10] };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 10);
    assert_eq!(bytes, vec![0xAA; 10]);
}

#[test]
fn test_bit_position_addition() {
    // Test that bit positions are added, not subtracted or multiplied
    #[derive(BeBytes, Debug, PartialEq)]
    struct BitAddTest {
        #[bits(4)]
        first: u8,
        #[bits(4)]
        second: u8,
        #[bits(8)]
        third: u8,
    }

    // Total should be 4 + 4 + 8 = 16 bits = 2 bytes
    // If += is replaced with -= or *=, this will be wrong
    assert_eq!(BitAddTest::field_size(), 2);

    let test = BitAddTest {
        first: 0xF,
        second: 0xF,
        third: 0xFF,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 2);
    assert_eq!(bytes[0], 0xFF); // first + second
    assert_eq!(bytes[1], 0xFF); // third
}

#[test]
fn test_byte_boundary_arithmetic() {
    // Test div_ceil arithmetic for byte boundaries
    #[derive(BeBytes, Debug, PartialEq)]
    struct ByteBoundaryTest {
        #[bits(7)]
        seven: u8,
        #[bits(9)]
        nine: u16,
    }

    // 7 + 9 = 16 bits = 2 bytes
    // This tests that div_ceil(16, 8) = 2 works correctly
    assert_eq!(ByteBoundaryTest::field_size(), 2);

    let test = ByteBoundaryTest {
        seven: 0x7F,
        nine: 0x1FF,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 2);
}

#[test]
fn test_field_type_determination() {
    // Test that arrays and primitives are properly identified
    #[derive(BeBytes, Debug, PartialEq)]
    struct TypeDetectionTest {
        array_field: [u8; 5],
        primitive_field: u32,
    }

    assert_eq!(TypeDetectionTest::field_size(), 5 + 4); // 9 bytes

    let test = TypeDetectionTest {
        array_field: [1, 2, 3, 4, 5],
        primitive_field: 0x12345678,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 9);

    // Verify parsing works
    let (parsed, consumed) = TypeDetectionTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, 9);
    assert_eq!(parsed, test);
}

#[test]
fn test_enum_size_calculation() {
    // Test that enum size calculation works correctly
    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
    enum TestEnum {
        A = 0,
        B = 127,
        C = 255,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct EnumSizeTest {
        value: TestEnum,
    }

    assert_eq!(EnumSizeTest::field_size(), 1);

    // Test all variants to ensure discriminant handling works
    for variant in [TestEnum::A, TestEnum::B, TestEnum::C] {
        let test = EnumSizeTest { value: variant };
        let bytes = test.to_be_bytes();
        assert_eq!(bytes.len(), 1);

        let (parsed, _) = EnumSizeTest::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value as u8, variant as u8);
    }
}

#[test]
fn test_bit_shift_operations() {
    // Test that bit shifts use << not >>
    #[derive(BeBytes, Debug, PartialEq)]
    struct BitShiftTest {
        #[bits(1)]
        bit0: u8,
        #[bits(1)]
        bit1: u8,
        #[bits(1)]
        bit2: u8,
        #[bits(1)]
        bit3: u8,
        #[bits(1)]
        bit4: u8,
        #[bits(1)]
        bit5: u8,
        #[bits(1)]
        bit6: u8,
        #[bits(1)]
        bit7: u8,
    }

    let test = BitShiftTest {
        bit0: 1,
        bit1: 0,
        bit2: 1,
        bit3: 0,
        bit4: 1,
        bit5: 0,
        bit6: 1,
        bit7: 0,
    };

    let bytes = test.to_be_bytes();
    // Should be 10101010 = 0xAA
    assert_eq!(bytes[0], 0xAA);

    // Parse back and verify each bit
    let (parsed, _) = BitShiftTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.bit0, 1);
    assert_eq!(parsed.bit1, 0);
    assert_eq!(parsed.bit2, 1);
    assert_eq!(parsed.bit3, 0);
    assert_eq!(parsed.bit4, 1);
    assert_eq!(parsed.bit5, 0);
    assert_eq!(parsed.bit6, 1);
    assert_eq!(parsed.bit7, 0);
}

#[test]
fn test_boundary_conditions() {
    // Test >= vs > and <= vs < conditions
    #[derive(BeBytes, Debug, PartialEq)]
    struct BoundaryConditionTest {
        #[bits(8)]
        exactly_one_byte: u8,
        #[bits(16)]
        exactly_two_bytes: u16,
    }

    assert_eq!(BoundaryConditionTest::field_size(), 3);

    let test = BoundaryConditionTest {
        exactly_one_byte: 255,
        exactly_two_bytes: 65535,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 3);

    // Values should be preserved exactly
    let (parsed, _) = BoundaryConditionTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.exactly_one_byte, 255);
    assert_eq!(parsed.exactly_two_bytes, 65535);
}

#[test]
fn test_attribute_parsing() {
    // Test that bit attributes are parsed correctly
    #[derive(BeBytes, Debug, PartialEq)]
    struct AttributeTest {
        #[bits(5)]
        five_bit_field: u8,
        #[bits(3)]
        three_bit_field: u8,
        #[bits(16)]
        sixteen_bit_field: u16,
    }

    let test = AttributeTest {
        five_bit_field: 31, // max 5-bit value
        three_bit_field: 7, // max 3-bit value
        sixteen_bit_field: 0xFFFF,
    };

    assert_eq!(AttributeTest::field_size(), 3); // 5 + 3 + 16 = 24 bits = 3 bytes

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 3);

    // Parse back and verify values
    let (parsed, _) = AttributeTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.five_bit_field, 31);
    assert_eq!(parsed.three_bit_field, 7);
    assert_eq!(parsed.sixteen_bit_field, 0xFFFF);
}
