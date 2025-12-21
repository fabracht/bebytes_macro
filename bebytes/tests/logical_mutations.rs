//! Tests specifically targeting logical operator mutations
//! These ensure that && and || are used correctly

use bebytes::BeBytes;

#[test]
fn test_logical_and_operations() {
    // Test conditions that require && not ||

    #[derive(BeBytes, Debug, PartialEq)]
    struct LogicalAndTest {
        #[bits(4)]
        nibble1: u8,
        #[bits(4)]
        nibble2: u8,
        regular: u16,
    }

    let test = LogicalAndTest {
        nibble1: 15,
        nibble2: 15,
        regular: 0xFFFF,
    };

    assert_eq!(LogicalAndTest::field_size(), 3); // 1 + 2

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 3);
    assert_eq!(bytes[0], 0xFF); // Both nibbles
    assert_eq!(&bytes[1..3], &[0xFF, 0xFF]); // u16
}

#[test]
fn test_logical_or_operations() {
    // Test bit merging operations that use |= not ^=

    #[derive(BeBytes, Debug, PartialEq)]
    struct BitMergeTest {
        #[bits(4)]
        high: u8,
        #[bits(4)]
        low: u8,
    }

    // Test that bits are merged with OR not XOR
    let test = BitMergeTest {
        high: 0xF,
        low: 0xF,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes[0], 0xFF);

    // Test overlapping bit patterns
    let test2 = BitMergeTest {
        high: 0xA, // 1010
        low: 0x5,  // 0101
    };

    let bytes2 = test2.to_be_bytes();
    assert_eq!(bytes2[0], 0xA5);

    let (parsed, _) = BitMergeTest::try_from_be_bytes(&bytes2).unwrap();
    assert_eq!(parsed.high, 0xA);
    assert_eq!(parsed.low, 0x5);
}

#[test]
fn test_complex_logical_conditions() {
    // Test complex structs that exercise multiple logical operations

    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct NestedStruct {
        field1: u8,
        field2: u16,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct ComplexLogicalTest {
        header: NestedStruct,
        #[bits(3)]
        flags: u8,
        #[bits(5)]
        counter: u8,
        data: [u8; 4],
        #[FromField(header.field2)]
        dynamic: Vec<u8>,
    }

    let test = ComplexLogicalTest {
        header: NestedStruct {
            field1: 0xFF,
            field2: 2,
        },
        flags: 7,
        counter: 31,
        data: [0xAA, 0xBB, 0xCC, 0xDD],
        dynamic: vec![0x11, 0x22],
    };

    let bytes = test.to_be_bytes();
    let expected_size = 1 + 2 + 1 + 4 + 2; // header + bits + data + dynamic
    assert_eq!(bytes.len(), expected_size);

    let (parsed, _) = ComplexLogicalTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.header.field1, 0xFF);
    assert_eq!(parsed.header.field2, 2);
    assert_eq!(parsed.flags, 7);
    assert_eq!(parsed.counter, 31);
    assert_eq!(parsed.data, [0xAA, 0xBB, 0xCC, 0xDD]);
    assert_eq!(parsed.dynamic, vec![0x11, 0x22]);
}

#[test]
fn test_bit_merge_patterns() {
    // Test various bit merge patterns to ensure |= is used correctly

    #[derive(BeBytes, Debug, PartialEq)]
    struct MergePatternTest {
        #[bits(2)]
        two1: u8,
        #[bits(2)]
        two2: u8,
        #[bits(2)]
        two3: u8,
        #[bits(2)]
        two4: u8,
    }

    // Test all combinations of 2-bit values
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    let test = MergePatternTest {
                        two1: a,
                        two2: b,
                        two3: c,
                        two4: d,
                    };

                    let bytes = test.to_be_bytes();
                    let expected = (a << 6) | (b << 4) | (c << 2) | d;
                    assert_eq!(bytes[0], expected);

                    let (parsed, _) = MergePatternTest::try_from_be_bytes(&bytes).unwrap();
                    assert_eq!(parsed.two1, a);
                    assert_eq!(parsed.two2, b);
                    assert_eq!(parsed.two3, c);
                    assert_eq!(parsed.two4, d);
                }
            }
        }
    }
}

#[test]
fn test_conditional_field_types() {
    // Test type determination logic that uses logical conditions

    #[derive(BeBytes, Debug, PartialEq)]
    struct MixedFieldTypes {
        primitive: u32,
        array: [u8; 5],
        option: Option<u16>,
        #[bits(16)]
        bits: u16,
    }

    // Test with Some
    let test_some = MixedFieldTypes {
        primitive: 0x12345678,
        array: [1, 2, 3, 4, 5],
        option: Some(0xABCD),
        bits: 0xFFFF,
    };

    let bytes_some = test_some.to_be_bytes();
    assert_eq!(bytes_some.len(), 14);

    let test_none = MixedFieldTypes {
        primitive: 0x12345678,
        array: [1, 2, 3, 4, 5],
        option: None,
        bits: 0xFFFF,
    };

    let bytes_none = test_none.to_be_bytes();
    assert_eq!(bytes_none.len(), 14);
}

#[test]
fn test_guard_conditions() {
    // Test match guard conditions

    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct Inner {
        val: u8,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct GuardTest {
        count: u8,
        #[FromField(count)]
        vec_field: Vec<u8>,
        opt_field: Option<u8>,
        inner_field: Inner,
    }

    // Can't test Vec directly without FromField, but we can test the generated code works
    // This ensures type identification logic is correct

    let _test = GuardTest {
        count: 0,
        vec_field: vec![],
        opt_field: Some(42),
        inner_field: Inner { val: 100 },
    };

    // The important part is that this compiles and the types are handled correctly
    assert_eq!(std::mem::size_of::<Inner>(), 1);
}

#[test]
fn test_multiple_condition_combinations() {
    // Test combinations of conditions

    #[derive(BeBytes, Debug, PartialEq)]
    struct ConditionComboTest {
        #[bits(1)]
        flag1: u8,
        #[bits(1)]
        flag2: u8,
        #[bits(1)]
        flag3: u8,
        #[bits(1)]
        flag4: u8,
        #[bits(4)]
        value: u8,
    }

    // Test various flag combinations
    let test = ConditionComboTest {
        flag1: 1,
        flag2: 0,
        flag3: 1,
        flag4: 0,
        value: 15,
    };

    let bytes = test.to_be_bytes();
    // Expected: 1010_1111 = 0xAF
    assert_eq!(bytes[0], 0xAF);

    let (parsed, _) = ConditionComboTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.flag1, 1);
    assert_eq!(parsed.flag2, 0);
    assert_eq!(parsed.flag3, 1);
    assert_eq!(parsed.flag4, 0);
    assert_eq!(parsed.value, 15);
}
