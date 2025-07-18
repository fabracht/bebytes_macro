use bebytes::BeBytes;
use test_case::test_case;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use core::fmt::Write;

// This file contains parameterized tests for various edge cases and complex scenarios
// Individual feature tests have been moved to unit/ subdirectories

#[derive(BeBytes, Debug, PartialEq)]
struct ComplexStruct {
    #[bits(3)]
    flag1: u8,
    #[bits(5)]
    flag2: u8,
    regular_field: u16,
    #[With(size(4))]
    fixed_vec: Vec<u8>,
    trailer: u32,
}

#[test_case(ComplexStruct { flag1: 7, flag2: 31, regular_field: 65535, fixed_vec: vec![1, 2, 3, 4], trailer: 0xDEADBEEF },
            vec![0xFF, 0xFF, 0xFF, 1, 2, 3, 4, 0xDE, 0xAD, 0xBE, 0xEF];
            "complex struct with all fields maxed")]
#[test_case(ComplexStruct { flag1: 0, flag2: 0, regular_field: 0, fixed_vec: vec![0, 0, 0, 0], trailer: 0 },
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            "complex struct with all zeros")]
fn test_complex_struct_serialization(input: ComplexStruct, expected_bytes: Vec<u8>) {
    let bytes = input.to_be_bytes();
    assert_eq!(bytes, expected_bytes);

    let (deserialized, consumed) = ComplexStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(deserialized, input);
    assert_eq!(consumed, expected_bytes.len());
}

// Test for edge cases with bit manipulation across byte boundaries
#[derive(BeBytes, Debug, PartialEq)]
struct BitBoundaryTest {
    #[bits(7)]
    seven_bits: u8,
    #[bits(9)]
    nine_bits: u16, // Spans two bytes
    #[bits(3)]
    three_bits: u8,
    #[bits(5)]
    five_bits: u8,
}

#[test_case(BitBoundaryTest { seven_bits: 127, nine_bits: 511, three_bits: 7, five_bits: 31 };
            "all bits set to max")]
#[test_case(BitBoundaryTest { seven_bits: 0, nine_bits: 0, three_bits: 0, five_bits: 0 };
            "all bits zero")]
#[test_case(BitBoundaryTest { seven_bits: 85, nine_bits: 341, three_bits: 5, five_bits: 21 };
            "alternating bit pattern")]
fn test_bit_boundary_handling(input: BitBoundaryTest) {
    let bytes = input.to_be_bytes();
    let (deserialized, _) = BitBoundaryTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(deserialized, input);

    // Also test little endian
    let le_bytes = input.to_le_bytes();
    let (le_deserialized, _) = BitBoundaryTest::try_from_le_bytes(&le_bytes).unwrap();
    assert_eq!(le_deserialized, input);
}
