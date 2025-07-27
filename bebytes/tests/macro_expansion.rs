//! Tests to verify the derive macro actually generates the expected code
//! This addresses the critical gap where derive_be_bytes could return Default::default()

use bebytes::BeBytes;

#[test]
fn test_derive_generates_trait_impl() {
    // This struct should have BeBytes trait methods after derive
    #[derive(BeBytes, Debug, PartialEq)]
    struct TestStruct {
        field1: u32,
        field2: u16,
    }

    // Verify the trait is implemented by calling its methods
    let test = TestStruct {
        field1: 0x12345678,
        field2: 0xABCD,
    };

    // These method calls prove the derive macro generated code
    let _size = TestStruct::field_size();
    let be_bytes = test.to_be_bytes();
    let le_bytes = test.to_le_bytes();

    // Verify the methods actually work
    assert_eq!(TestStruct::field_size(), 6); // 4 + 2 bytes
    assert_eq!(be_bytes.len(), 6);
    assert_eq!(le_bytes.len(), 6);

    // Verify parsing works
    let (parsed_be, consumed_be) = TestStruct::try_from_be_bytes(&be_bytes).unwrap();
    assert_eq!(parsed_be, test);
    assert_eq!(consumed_be, 6);

    let (parsed_le, consumed_le) = TestStruct::try_from_le_bytes(&le_bytes).unwrap();
    assert_eq!(parsed_le, test);
    assert_eq!(consumed_le, 6);
}

#[test]
fn test_derive_with_bit_fields() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct BitFieldStruct {
        #[bits(4)]
        nibble: u8,
        #[bits(12)]
        twelve_bits: u16,
        regular: u32,
    }

    let test = BitFieldStruct {
        nibble: 0xF,
        twelve_bits: 0xFFF,
        regular: 0xDEADBEEF,
    };

    // Verify methods exist and work
    assert_eq!(BitFieldStruct::field_size(), 6); // 2 + 4 bytes
    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 6);

    let (parsed, _) = BitFieldStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, test);
}

#[test]
fn test_derive_with_enums() {
    #[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
    #[repr(u8)]
    enum TestEnum {
        First = 1,
        Second = 2,
        Third = 3,
    }

    // Verify enum derive works
    let val = TestEnum::Second;
    let bytes = val.to_be_bytes();
    assert_eq!(bytes, vec![2]);

    let (parsed, _) = TestEnum::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, TestEnum::Second);
}

#[test]
fn test_derive_with_vectors() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct VectorStruct {
        len: u16,
        #[FromField(len)]
        data: Vec<u8>,
    }

    let test = VectorStruct {
        len: 3,
        data: vec![1, 2, 3],
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 5); // 2 + 3

    let (parsed, _) = VectorStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, test);
}

#[test]
fn test_derive_generates_new_constructor() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct ConstructorTest {
        a: u32,
        b: u16,
        c: u8,
    }

    // The derive macro should generate a new() constructor
    let test = ConstructorTest::new(0x12345678, 0xABCD, 0xEF);
    assert_eq!(test.a, 0x12345678);
    assert_eq!(test.b, 0xABCD);
    assert_eq!(test.c, 0xEF);
}

#[test]
fn test_macro_generates_all_required_methods() {
    #[derive(BeBytes)]
    struct MethodTest {
        value: u32,
    }

    // Verify all required trait methods exist by using them
    let test = MethodTest { value: 42 };

    // Static method
    let size = MethodTest::field_size();
    assert!(size > 0);

    // Instance methods for serialization
    let be_bytes = test.to_be_bytes();
    let le_bytes = test.to_le_bytes();
    assert!(!be_bytes.is_empty());
    assert!(!le_bytes.is_empty());

    // Static methods for deserialization
    let be_result = MethodTest::try_from_be_bytes(&be_bytes);
    let le_result = MethodTest::try_from_le_bytes(&le_bytes);
    assert!(be_result.is_ok());
    assert!(le_result.is_ok());
}
