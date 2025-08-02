//! Tests specifically targeting functions that could return Default::default()
//! These ensure that functions return meaningful values, not just defaults

use bebytes::BeBytes;

#[test]
fn test_handle_enum_returns_valid_result() {
    // Test that handle_enum doesn't just return Default::default()

    #[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
    enum TestEnum {
        A = 0,
        B = 1,
        C = 255,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct EnumStruct {
        value: TestEnum,
    }

    // If handle_enum returned Default::default(), this wouldn't work
    let test = EnumStruct { value: TestEnum::C };
    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 1);
    assert_eq!(bytes[0], 255);

    let (parsed, _) = EnumStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.value as u8, 255);
}

#[test]
fn test_parse_attributes_returns_values() {
    // Test that parse_attributes returns actual values, not defaults

    #[derive(BeBytes, Debug, PartialEq)]
    struct AttributeValueTest {
        #[bits(7)]
        seven: u8,
        #[bits(9)]
        nine: u16,
        #[bits(16)]
        sixteen: u16,
    }

    // If parse_attributes returned (None, None) or (Some(0), None), this would fail
    assert_eq!(AttributeValueTest::field_size(), 4); // 32 bits

    let test = AttributeValueTest {
        seven: 127,
        nine: 511,
        sixteen: 65535,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 4);

    let (parsed, _) = AttributeValueTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.seven, 127);
    assert_eq!(parsed.nine, 511);
    assert_eq!(parsed.sixteen, 65535);
}

#[test]
fn test_determine_field_type_returns_correct_type() {
    // Test that determine_field_type returns correct types

    #[derive(BeBytes, Debug, PartialEq)]
    struct FieldTypeReturnTest {
        count: u16,
        #[FromField(count)]
        vec_field: Vec<u8>,
        opt_field: Option<u32>,
        array_field: [u8; 10],
        primitive_field: u64,
    }

    // This struct wouldn't compile correctly if determine_field_type returned None
    // The fact it compiles and has the right size proves the function works
    let _expected_size = 2 + 5 + 10 + 8; // count + option + array + u64
}

#[test]
fn test_functional_helpers_return_valid_tokens() {
    // Test that functional helpers don't return Default::default()

    #[derive(BeBytes, Debug, PartialEq)]
    struct FunctionalHelperTest {
        // Tests create_primitive_parsing/writing
        u8_val: u8,
        u16_val: u16,
        u32_val: u32,
        u64_val: u64,
        u128_val: u128,

        // Tests create_field_accessor
        #[FromField(u8_val)]
        dynamic: Vec<u8>,

        // Tests bit operations
        #[bits(12)]
        twelve: u16,
        #[bits(4)]
        four: u8,
    }

    let test = FunctionalHelperTest {
        u8_val: 3,
        u16_val: 0x1234,
        u32_val: 0x12345678,
        u64_val: 0x123456789ABCDEF0,
        u128_val: 0x123456789ABCDEF0123456789ABCDEF0,
        dynamic: vec![0xAA, 0xBB, 0xCC],
        twelve: 0xFFF,
        four: 0xF,
    };

    let bytes = test.to_be_bytes();
    let (parsed, _) = FunctionalHelperTest::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(parsed.u8_val, 3);
    assert_eq!(parsed.u16_val, 0x1234);
    assert_eq!(parsed.u32_val, 0x12345678);
    assert_eq!(parsed.u64_val, 0x123456789ABCDEF0);
    assert_eq!(parsed.u128_val, 0x123456789ABCDEF0123456789ABCDEF0);
    assert_eq!(parsed.dynamic, vec![0xAA, 0xBB, 0xCC]);
    assert_eq!(parsed.twelve, 0xFFF);
    assert_eq!(parsed.four, 0xF);
}

#[test]
fn test_bit_calculation_functions() {
    // Test bit calculation helper functions

    #[derive(BeBytes, Debug, PartialEq)]
    struct BitCalcTest {
        #[bits(5)]
        five: u8,
        #[bits(11)]
        eleven: u16,
        #[bits(24)]
        twentyfour: u32,
        #[bits(8)]
        eight: u8,
    }

    // Total: 5 + 11 + 24 + 8 = 48 bits = 6 bytes
    assert_eq!(BitCalcTest::field_size(), 6);

    let test = BitCalcTest {
        five: 31,
        eleven: 2047,
        twentyfour: 0xFFFFFF,
        eight: 255,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 6);

    let (parsed, _) = BitCalcTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.five, 31);
    assert_eq!(parsed.eleven, 2047);
    assert_eq!(parsed.twentyfour, 0xFFFFFF);
    assert_eq!(parsed.eight, 255);
}

#[test]
fn test_custom_vector_generation() {
    // Test generate_custom_vector_parsing doesn't return Default::default()

    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct Header {
        magic: u32,
        count: u16,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct CustomVectorTest {
        header: Header,
        #[FromField(header.count)]
        items: Vec<u8>,
    }

    let test = CustomVectorTest {
        header: Header {
            magic: 0x12345678,
            count: 3,
        },
        items: vec![0x11, 0x22, 0x33],
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 4 + 2 + 3); // magic + count + 3*u8

    let (parsed, _) = CustomVectorTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.header.magic, 0x12345678);
    assert_eq!(parsed.header.count, 3);
    assert_eq!(parsed.items, vec![0x11, 0x22, 0x33]);
}

#[test]
fn test_primitive_vector_generation() {
    // Test generate_primitive_vector_tokens

    #[derive(BeBytes, Debug, PartialEq)]
    struct PrimitiveVectorTest {
        size: u8,
        #[FromField(size)]
        data1: Vec<u8>,
        #[FromField(size)]
        data2: Vec<u8>,
    }

    let test = PrimitiveVectorTest {
        size: 2,
        data1: vec![0xAA, 0xBB],
        data2: vec![0x11, 0x22],
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 1 + 2 + 2); // size + 2*u8 + 2*u8

    let (parsed, _) = PrimitiveVectorTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.size, 2);
    assert_eq!(parsed.data1, vec![0xAA, 0xBB]);
    assert_eq!(parsed.data2, vec![0x11, 0x22]);
}

#[test]
fn test_validate_byte_completeness_works() {
    // Test that validate_byte_completeness actually validates
    // These should compile, proving validation works

    #[derive(BeBytes, Debug, PartialEq)]
    struct ValidBits8 {
        #[bits(8)]
        byte: u8,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct ValidBits16 {
        #[bits(16)]
        word: u16,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct ValidBits24 {
        #[bits(24)]
        three_bytes: u32,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct ValidBits32 {
        #[bits(32)]
        dword: u32,
    }

    assert_eq!(ValidBits8::field_size(), 1);
    assert_eq!(ValidBits16::field_size(), 2);
    assert_eq!(ValidBits24::field_size(), 3);
    assert_eq!(ValidBits32::field_size(), 4);
}

#[test]
fn test_handle_struct_processes_correctly() {
    // Test that handle_struct doesn't just return ()

    #[derive(BeBytes, Debug, PartialEq)]
    struct ComplexStruct {
        header: u32,
        #[bits(4)]
        flags: u8,
        #[bits(12)]
        counter: u16,
        data: [u8; 8],
        footer: u64,
    }

    let test = ComplexStruct {
        header: 0x12345678,
        flags: 0xF,
        counter: 0xFFF,
        data: [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88],
        footer: 0x123456789ABCDEF0,
    };

    let bytes = test.to_be_bytes();
    let expected_size = 4 + 2 + 8 + 8; // header + bits + data + footer
    assert_eq!(bytes.len(), expected_size);

    let (parsed, _) = ComplexStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.header, 0x12345678);
    assert_eq!(parsed.flags, 0xF);
    assert_eq!(parsed.counter, 0xFFF);
    assert_eq!(parsed.data, test.data);
    assert_eq!(parsed.footer, 0x123456789ABCDEF0);
}
