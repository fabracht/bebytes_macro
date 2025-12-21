//! Tests specifically targeting comparison operator mutations
//! These ensure that ==, !=, <, >, <=, >= are used correctly

use bebytes::BeBytes;

#[test]
fn test_vec_type_identification() {
    // Tests that Vec is identified with == not !=

    #[derive(BeBytes, Debug, PartialEq)]
    struct VecTest {
        count: u16,
        #[FromField(count)]
        data: Vec<u8>,
    }

    let test = VecTest {
        count: 5,
        data: vec![1, 2, 3, 4, 5],
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 7); // 2 + 5
    assert_eq!(&bytes[0..2], &[0, 5]);
    assert_eq!(&bytes[2..7], &[1, 2, 3, 4, 5]);

    let (parsed, _) = VecTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.count, 5);
    assert_eq!(parsed.data, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_option_type_identification() {
    // Tests that Option is identified with == not !=
    // Note: BeBytes has limited Option support, only for primitives

    #[derive(BeBytes, Debug, PartialEq)]
    struct OptionTest {
        header: u8,
        opt_value: Option<u32>,
        footer: u8,
    }

    // Test Some case
    let test_some = OptionTest {
        header: 0xAA,
        opt_value: Some(0x12345678),
        footer: 0xBB,
    };

    let bytes_some = test_some.to_be_bytes();
    assert_eq!(bytes_some.len(), 7);
    assert_eq!(bytes_some[0], 0xAA);
    assert_eq!(bytes_some[1], 0x01);
    assert_eq!(&bytes_some[2..6], &[0x12, 0x34, 0x56, 0x78]);
    assert_eq!(bytes_some[6], 0xBB);
}

#[test]
fn test_bit_limit_comparisons() {
    // Tests > vs < vs >= vs <= in bit limit checks

    #[derive(BeBytes, Debug, PartialEq)]
    struct BitLimitCompare {
        #[bits(4)]
        nibble: u8,
        #[bits(12)]
        twelve: u16,
    }

    // Test exact limit values
    let test = BitLimitCompare {
        nibble: 15,   // Exactly 2^4 - 1
        twelve: 4095, // Exactly 2^12 - 1
    };

    let bytes = test.to_be_bytes();
    let (parsed, _) = BitLimitCompare::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.nibble, 15);
    assert_eq!(parsed.twelve, 4095);

    // Test one below limit
    let test2 = BitLimitCompare {
        nibble: 14,
        twelve: 4094,
    };

    let bytes2 = test2.to_be_bytes();
    let (parsed2, _) = BitLimitCompare::try_from_be_bytes(&bytes2).unwrap();
    assert_eq!(parsed2.nibble, 14);
    assert_eq!(parsed2.twelve, 4094);

    // Test zero values
    let test3 = BitLimitCompare {
        nibble: 0,
        twelve: 0,
    };

    let bytes3 = test3.to_be_bytes();
    let (parsed3, _) = BitLimitCompare::try_from_be_bytes(&bytes3).unwrap();
    assert_eq!(parsed3.nibble, 0);
    assert_eq!(parsed3.twelve, 0);
}

#[test]
fn test_enum_discriminant_comparison() {
    // Test enum discriminant boundary checks

    #[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
    enum BoundaryEnum {
        Zero = 0,
        One = 1,
        Two = 2,
        Max = 255,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct EnumCompareTest {
        value: BoundaryEnum,
    }

    // Test boundary values
    let boundary_values = [
        BoundaryEnum::Zero,
        BoundaryEnum::One,
        BoundaryEnum::Two,
        BoundaryEnum::Max,
    ];

    for &value in &boundary_values {
        let test = EnumCompareTest { value };
        let bytes = test.to_be_bytes();
        assert_eq!(bytes.len(), 1);

        let (parsed, _) = EnumCompareTest::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value as u8, value as u8);
    }
}

#[test]
fn test_array_length_comparison() {
    // Test array length checks

    #[derive(BeBytes, Debug, PartialEq)]
    struct ArrayCompareTest {
        #[bits(8)]
        prefix: u8,
        data: [u8; 0], // Zero-length array
        #[bits(8)]
        suffix: u8,
    }

    let test = ArrayCompareTest {
        prefix: 0xAA,
        data: [],
        suffix: 0xBB,
    };

    assert_eq!(ArrayCompareTest::field_size(), 2); // 1 + 0 + 1

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 2);
    assert_eq!(bytes[0], 0xAA);
    assert_eq!(bytes[1], 0xBB);
}

#[test]
fn test_field_access_equality() {
    // Test field access path generation with == comparisons

    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct Header {
        version: u8,
        length: u16,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct FieldAccessTest {
        header: Header,
        #[FromField(header.length)]
        payload: Vec<u8>,
    }

    let test = FieldAccessTest {
        header: Header {
            version: 1,
            length: 3,
        },
        payload: vec![0xAA, 0xBB, 0xCC],
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 6); // 1 + 2 + 3

    let (parsed, _) = FieldAccessTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.header.version, 1);
    assert_eq!(parsed.header.length, 3);
    assert_eq!(parsed.payload.len(), 3);
}

#[test]
fn test_bit_position_boundaries() {
    // Test bit position calculations at boundaries

    #[derive(BeBytes, Debug, PartialEq)]
    struct BitBoundaryTest {
        #[bits(7)]
        almost_byte: u8,
        #[bits(1)]
        one_bit: u8,
        #[bits(8)]
        full_byte: u8,
        #[bits(15)]
        almost_two: u16,
        #[bits(1)]
        one_more: u8,
    }

    assert_eq!(BitBoundaryTest::field_size(), 4); // 32 bits total

    let test = BitBoundaryTest {
        almost_byte: 127,
        one_bit: 1,
        full_byte: 255,
        almost_two: 32767,
        one_more: 1,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 4);

    let (parsed, _) = BitBoundaryTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.almost_byte, 127);
    assert_eq!(parsed.one_bit, 1);
    assert_eq!(parsed.full_byte, 255);
    assert_eq!(parsed.almost_two, 32767);
    assert_eq!(parsed.one_more, 1);
}

#[test]
fn test_primitive_type_checks() {
    // Test primitive type identification

    #[derive(BeBytes, Debug, PartialEq)]
    struct PrimitiveCheckTest {
        u8_val: u8,
        i8_val: i8,
        u16_val: u16,
        i16_val: i16,
        u32_val: u32,
        i32_val: i32,
        u64_val: u64,
        i64_val: i64,
        u128_val: u128,
        i128_val: i128,
    }

    let test = PrimitiveCheckTest {
        u8_val: 255,
        i8_val: -1,
        u16_val: 65535,
        i16_val: -1,
        u32_val: 0xFFFFFFFF,
        i32_val: -1,
        u64_val: 0xFFFFFFFFFFFFFFFF,
        i64_val: -1,
        u128_val: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        i128_val: -1,
    };

    let bytes = test.to_be_bytes();
    let expected_size = 1 + 1 + 2 + 2 + 4 + 4 + 8 + 8 + 16 + 16;
    assert_eq!(bytes.len(), expected_size);

    let (parsed, _) = PrimitiveCheckTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.u8_val, 255);
    assert_eq!(parsed.i8_val, -1);
    assert_eq!(parsed.u16_val, 65535);
    assert_eq!(parsed.i16_val, -1);
}

#[test]
fn test_non_empty_path_segments() {
    // Test path segment empty checks

    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct InnerType {
        value: u32,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct PathSegmentTest {
        inner: InnerType,
        primitive: u16,
        array: [u8; 3],
    }

    let test = PathSegmentTest {
        inner: InnerType { value: 0x12345678 },
        primitive: 0xABCD,
        array: [1, 2, 3],
    };

    assert_eq!(PathSegmentTest::field_size(), 4 + 2 + 3);

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 9);

    let (parsed, _) = PathSegmentTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.inner.value, 0x12345678);
    assert_eq!(parsed.primitive, 0xABCD);
    assert_eq!(parsed.array, [1, 2, 3]);
}
