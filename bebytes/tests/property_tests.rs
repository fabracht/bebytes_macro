//! Property-based tests for round-trip serialization/deserialization
//! These tests ensure that for any valid input, serialize then deserialize produces the same value

use bebytes::BeBytes;

#[test]
fn test_primitive_round_trips() {
    // Test that primitives BeBytes implementations work correctly through structs

    // Test all primitive types
    #[derive(BeBytes, Debug, PartialEq)]
    struct AllPrimitives {
        u8_val: u8,
        u16_val: u16,
        u32_val: u32,
        u64_val: u64,
        u128_val: u128,
        i8_val: i8,
        i16_val: i16,
        i32_val: i32,
        i64_val: i64,
        i128_val: i128,
    }

    let test_cases = vec![
        // All zeros
        AllPrimitives {
            u8_val: 0,
            u16_val: 0,
            u32_val: 0,
            u64_val: 0,
            u128_val: 0,
            i8_val: 0,
            i16_val: 0,
            i32_val: 0,
            i64_val: 0,
            i128_val: 0,
        },
        // All max values
        AllPrimitives {
            u8_val: u8::MAX,
            u16_val: u16::MAX,
            u32_val: u32::MAX,
            u64_val: u64::MAX,
            u128_val: u128::MAX,
            i8_val: i8::MAX,
            i16_val: i16::MAX,
            i32_val: i32::MAX,
            i64_val: i64::MAX,
            i128_val: i128::MAX,
        },
        // Min signed values
        AllPrimitives {
            u8_val: 0,
            u16_val: 0,
            u32_val: 0,
            u64_val: 0,
            u128_val: 0,
            i8_val: i8::MIN,
            i16_val: i16::MIN,
            i32_val: i32::MIN,
            i64_val: i64::MIN,
            i128_val: i128::MIN,
        },
        // Mixed values
        AllPrimitives {
            u8_val: 0x12,
            u16_val: 0x3456,
            u32_val: 0x789ABCDE,
            u64_val: 0xFEDCBA9876543210,
            u128_val: 0x123456789ABCDEF0123456789ABCDEF0,
            i8_val: -42,
            i16_val: -1234,
            i32_val: -123456789,
            i64_val: -1234567890123456,
            i128_val: -12345678901234567890123456789012345,
        },
    ];

    for original in test_cases {
        let be_bytes = original.to_be_bytes();
        let le_bytes = original.to_le_bytes();

        let (be_parsed, be_consumed) = AllPrimitives::try_from_be_bytes(&be_bytes).unwrap();
        let (le_parsed, le_consumed) = AllPrimitives::try_from_le_bytes(&le_bytes).unwrap();

        assert_eq!(be_parsed, original);
        assert_eq!(le_parsed, original);
        assert_eq!(be_consumed, be_bytes.len());
        assert_eq!(le_consumed, le_bytes.len());
    }
}

#[test]
fn test_struct_round_trips() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct TestStruct {
        a: u32,
        b: i16,
        c: u8,
        d: i8,
    }

    // Test various combinations
    let test_cases = vec![
        TestStruct {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
        },
        TestStruct {
            a: u32::MAX,
            b: i16::MAX,
            c: u8::MAX,
            d: i8::MAX,
        },
        TestStruct {
            a: u32::MIN,
            b: i16::MIN,
            c: u8::MIN,
            d: i8::MIN,
        },
        TestStruct {
            a: 12345,
            b: -1234,
            c: 42,
            d: -42,
        },
        TestStruct {
            a: 0xDEADBEEF,
            b: 0x1234,
            c: 0xAB,
            d: -0x12,
        },
    ];

    for original in test_cases {
        let be_bytes = original.to_be_bytes();
        let le_bytes = original.to_le_bytes();

        let (be_parsed, be_consumed) = TestStruct::try_from_be_bytes(&be_bytes).unwrap();
        let (le_parsed, le_consumed) = TestStruct::try_from_le_bytes(&le_bytes).unwrap();

        assert_eq!(be_parsed, original);
        assert_eq!(le_parsed, original);
        assert_eq!(be_consumed, TestStruct::field_size());
        assert_eq!(le_consumed, TestStruct::field_size());
    }
}

#[test]
fn test_bit_field_round_trips() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct BitFieldStruct {
        #[bits(3)]
        three: u8,
        #[bits(5)]
        five: u8,
        #[bits(12)]
        twelve: u16,
        #[bits(4)]
        four: u8,
    }

    // Test all valid combinations for small bit fields
    for three in 0..8 {
        for five in 0..32 {
            for four in 0..16 {
                let test = BitFieldStruct {
                    three: three as u8,
                    five: five as u8,
                    twelve: 0xFFF, // Max value for 12 bits
                    four: four as u8,
                };

                let be_bytes = test.to_be_bytes();
                let le_bytes = test.to_le_bytes();

                let (be_parsed, _) = BitFieldStruct::try_from_be_bytes(&be_bytes).unwrap();
                let (le_parsed, _) = BitFieldStruct::try_from_le_bytes(&le_bytes).unwrap();

                assert_eq!(be_parsed, test);
                assert_eq!(le_parsed, test);
            }
        }
    }
}

#[test]
fn test_array_round_trips() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct ArrayStruct {
        header: u16,
        data: [u8; 16],
        footer: u32,
    }

    // Test patterns
    let patterns = vec![
        [0u8; 16],
        [0xFF; 16],
        [0xAA; 16],
        [0x55; 16],
        (0..16)
            .map(|i| i as u8)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
        (0..16)
            .map(|i| (255 - i) as u8)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    ];

    for pattern in patterns {
        let test = ArrayStruct {
            header: 0x1234,
            data: pattern,
            footer: 0xDEADBEEF,
        };

        let be_bytes = test.to_be_bytes();
        let le_bytes = test.to_le_bytes();

        let (be_parsed, _) = ArrayStruct::try_from_be_bytes(&be_bytes).unwrap();
        let (le_parsed, _) = ArrayStruct::try_from_le_bytes(&le_bytes).unwrap();

        assert_eq!(be_parsed, test);
        assert_eq!(le_parsed, test);
    }
}

#[test]
fn test_nested_struct_round_trips() {
    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct Inner {
        x: u16,
        y: u16,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct Outer {
        id: u32,
        inner: Inner,
        flags: u8,
    }

    let test_cases = vec![
        Outer {
            id: 0,
            inner: Inner { x: 0, y: 0 },
            flags: 0,
        },
        Outer {
            id: u32::MAX,
            inner: Inner {
                x: u16::MAX,
                y: u16::MAX,
            },
            flags: u8::MAX,
        },
        Outer {
            id: 0x12345678,
            inner: Inner {
                x: 0x1234,
                y: 0x5678,
            },
            flags: 0b10101010,
        },
    ];

    for original in test_cases {
        let be_bytes = original.to_be_bytes();
        let le_bytes = original.to_le_bytes();

        let (be_parsed, _) = Outer::try_from_be_bytes(&be_bytes).unwrap();
        let (le_parsed, _) = Outer::try_from_le_bytes(&le_bytes).unwrap();

        assert_eq!(be_parsed, original);
        assert_eq!(le_parsed, original);
    }
}

#[test]
fn test_mixed_sizes_round_trips() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct MixedSizes {
        tiny: u8,
        small: u16,
        medium: u32,
        large: u64,
        huge: u128,
    }

    // Test various size combinations
    let test_cases = vec![
        MixedSizes {
            tiny: 0,
            small: 0,
            medium: 0,
            large: 0,
            huge: 0,
        },
        MixedSizes {
            tiny: u8::MAX,
            small: u16::MAX,
            medium: u32::MAX,
            large: u64::MAX,
            huge: u128::MAX,
        },
        MixedSizes {
            tiny: 0x12,
            small: 0x3456,
            medium: 0x789ABCDE,
            large: 0xFEDCBA9876543210,
            huge: 0x123456789ABCDEF0123456789ABCDEF0,
        },
    ];

    for original in test_cases {
        let be_bytes = original.to_be_bytes();
        let le_bytes = original.to_le_bytes();

        let (be_parsed, _) = MixedSizes::try_from_be_bytes(&be_bytes).unwrap();
        let (le_parsed, _) = MixedSizes::try_from_le_bytes(&le_bytes).unwrap();

        assert_eq!(be_parsed, original);
        assert_eq!(le_parsed, original);
    }
}

#[test]
fn test_extreme_bit_combinations() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct ExtremeBits {
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
        #[bits(1)]
        bit8: u8,
        regular: u32,
        #[bits(24)]
        three_bytes: u32,
        #[bits(8)]
        one_byte: u8,
    }

    // Test all single bit combinations
    for i in 0..=255u8 {
        let test = ExtremeBits {
            bit1: (i >> 7) & 1,
            bit2: (i >> 6) & 1,
            bit3: (i >> 5) & 1,
            bit4: (i >> 4) & 1,
            bit5: (i >> 3) & 1,
            bit6: (i >> 2) & 1,
            bit7: (i >> 1) & 1,
            bit8: i & 1,
            regular: 0xDEADBEEF,
            three_bytes: 0xFFFFFF,
            one_byte: 0xFF,
        };

        let be_bytes = test.to_be_bytes();
        let le_bytes = test.to_le_bytes();

        let (be_parsed, _) = ExtremeBits::try_from_be_bytes(&be_bytes).unwrap();
        let (le_parsed, _) = ExtremeBits::try_from_le_bytes(&le_bytes).unwrap();

        assert_eq!(be_parsed, test);
        assert_eq!(le_parsed, test);
    }
}

#[test]
fn test_enum_round_trips() {
    #[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
    #[repr(u8)]
    enum TestEnum {
        Zero = 0,
        One = 1,
        Two = 2,
        Max = 255,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct EnumStruct {
        before: u16,
        variant: TestEnum,
        after: u32,
    }

    let variants = vec![TestEnum::Zero, TestEnum::One, TestEnum::Two, TestEnum::Max];

    for variant in variants {
        let test = EnumStruct {
            before: 0x1234,
            variant,
            after: 0xABCDEF00,
        };

        let be_bytes = test.to_be_bytes();
        let le_bytes = test.to_le_bytes();

        let (be_parsed, _) = EnumStruct::try_from_be_bytes(&be_bytes).unwrap();
        let (le_parsed, _) = EnumStruct::try_from_le_bytes(&le_bytes).unwrap();

        assert_eq!(be_parsed, test);
        assert_eq!(le_parsed, test);
    }
}

#[test]
fn test_boundary_values() {
    // Test values at bit boundaries
    #[derive(BeBytes, Debug, PartialEq)]
    struct BoundaryTest {
        #[bits(4)]
        nibble: u8,
        #[bits(12)]
        twelve: u16,
        #[bits(16)]
        sixteen: u16,
        #[bits(20)]
        twenty: u32,
        #[bits(12)]
        twelve2: u16,
    }

    let boundary_values = vec![
        (0x0, 0x000, 0x0000, 0x00000, 0x000),
        (0xF, 0xFFF, 0xFFFF, 0xFFFFF, 0xFFF),
        (0x8, 0x800, 0x8000, 0x80000, 0x800),
        (0x7, 0x7FF, 0x7FFF, 0x7FFFF, 0x7FF),
        (0x1, 0x001, 0x0001, 0x00001, 0x001),
    ];

    for (nibble, twelve, sixteen, twenty, twelve2) in boundary_values {
        let test = BoundaryTest {
            nibble,
            twelve,
            sixteen,
            twenty,
            twelve2,
        };

        let be_bytes = test.to_be_bytes();
        let le_bytes = test.to_le_bytes();

        let (be_parsed, _) = BoundaryTest::try_from_be_bytes(&be_bytes).unwrap();
        let (le_parsed, _) = BoundaryTest::try_from_le_bytes(&le_bytes).unwrap();

        assert_eq!(be_parsed, test);
        assert_eq!(le_parsed, test);
    }
}
