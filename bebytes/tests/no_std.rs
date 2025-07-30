#![no_std]
#![cfg(not(feature = "std"))]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use bebytes::{BeBytes, BeBytesError};

#[derive(BeBytes, Debug, PartialEq)]
struct SimpleStruct {
    field1: u8,
    field2: u16,
    field3: u32,
}

#[derive(BeBytes, Debug, PartialEq)]
struct BitFieldStruct {
    #[bits(3)]
    flag1: u8,
    #[bits(5)]
    flag2: u8,
    value: u16,
}

#[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
enum TestEnum {
    Variant1 = 0,
    Variant2 = 1,
    Variant3 = 2,
}

#[derive(BeBytes, Debug, PartialEq)]
struct StructWithEnum {
    prefix: u8,
    variant: TestEnum,
    suffix: u16,
}

#[derive(BeBytes, Debug, PartialEq)]
struct VectorStruct {
    #[With(size(4))]
    data: Vec<u8>,
    tail: u16,
}

#[test]
fn test_simple_struct_no_std() {
    let original = SimpleStruct {
        field1: 0x42,
        field2: 0x1234,
        field3: 0xDEADBEEF,
    };

    let bytes = original.to_be_bytes();
    assert_eq!(bytes.len(), 7); // 1 + 2 + 4

    let (decoded, consumed) = SimpleStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, 7);
    assert_eq!(decoded, original);
}

#[test]
fn test_bitfield_struct_no_std() {
    let original = BitFieldStruct {
        flag1: 0b101,
        flag2: 0b11010,
        value: 0xABCD,
    };

    let bytes = original.to_be_bytes();
    assert_eq!(bytes.len(), 3); // 8 bits + 16 bits

    let (decoded, consumed) = BitFieldStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, 3);
    assert_eq!(decoded, original);
}

#[test]
fn test_enum_no_std() {
    let original = StructWithEnum {
        prefix: 0xFF,
        variant: TestEnum::Variant2,
        suffix: 0x5678,
    };

    let bytes = original.to_be_bytes();
    let (decoded, _) = StructWithEnum::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_vector_no_std() {
    let original = VectorStruct {
        data: vec![0xAA, 0xBB, 0xCC, 0xDD],
        tail: 0x9999,
    };

    let bytes = original.to_be_bytes();
    assert_eq!(bytes.len(), 6); // 4 + 2

    let (decoded, consumed) = VectorStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, 6);
    assert_eq!(decoded, original);
}

#[test]
fn test_error_handling_no_std() {
    // Test empty buffer
    let result = SimpleStruct::try_from_be_bytes(&[]);
    match result {
        Err(BeBytesError::EmptyBuffer) => {}
        _ => panic!("Expected EmptyBuffer error"),
    }

    // Test insufficient data
    let result = SimpleStruct::try_from_be_bytes(&[0x42, 0x12]); // Need 7 bytes
    match result {
        Err(BeBytesError::InsufficientData { expected, actual }) => {
            assert_eq!(expected, 2); // Expecting 2 bytes for u16 after u8
            assert_eq!(actual, 1); // Only 1 byte left
        }
        _ => panic!("Expected InsufficientData error"),
    }
}

#[test]
fn test_little_endian_no_std() {
    let original = SimpleStruct {
        field1: 0x42,
        field2: 0x1234,
        field3: 0xDEADBEEF,
    };

    let le_bytes = original.to_le_bytes();
    let be_bytes = original.to_be_bytes();

    // Verify endianness difference
    assert_ne!(le_bytes[1..3], be_bytes[1..3]); // u16 bytes should differ
    assert_ne!(le_bytes[3..7], be_bytes[3..7]); // u32 bytes should differ

    let (decoded, _) = SimpleStruct::try_from_le_bytes(&le_bytes).unwrap();
    assert_eq!(decoded, original);
}

// Test array support in no_std
#[derive(BeBytes, Debug, PartialEq)]
struct ArrayStruct {
    header: [u8; 4],
    data: u32,
}

#[test]
fn test_array_no_std() {
    let original = ArrayStruct {
        header: [0x01, 0x02, 0x03, 0x04],
        data: 0x12345678,
    };

    let bytes = original.to_be_bytes();
    assert_eq!(bytes.len(), 8);

    let (decoded, _) = ArrayStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded, original);
}

// Test that alloc::format works for error formatting
#[test]
fn test_error_formatting_no_std() {
    use alloc::format;

    let err = BeBytesError::InvalidDiscriminant {
        value: 42,
        type_name: "TestEnum",
    };
    let formatted = format!("{}", err);
    assert_eq!(formatted, "Invalid discriminant 42 for type TestEnum");
}
