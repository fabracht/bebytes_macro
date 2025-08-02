//! Tests specifically targeting arithmetic operation mutations
//! These tests ensure that arithmetic operations use the correct operators

use bebytes::BeBytes;

#[test]
fn test_div_ceil_implementation() {
    // Tests that div_ceil uses / not % or *
    // If / is replaced with %, these tests will fail

    #[derive(BeBytes, Debug, PartialEq)]
    struct DivCeilTest1 {
        #[bits(7)]
        seven: u8,
        #[bits(1)]
        one: u8,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct DivCeilTest2 {
        #[bits(15)]
        fifteen: u16,
        #[bits(1)]
        one: u8,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct DivCeilTest3 {
        #[bits(25)]
        twentyfive: u32,
        #[bits(7)]
        seven: u8,
    }

    // 7 + 1 = 8 bits = 1 byte
    assert_eq!(DivCeilTest1::field_size(), 1);

    // 15 + 1 = 16 bits = 2 bytes
    assert_eq!(DivCeilTest2::field_size(), 2);

    // 25 + 7 = 32 bits = 4 bytes
    assert_eq!(DivCeilTest3::field_size(), 4);

    // Test actual usage
    let test1 = DivCeilTest1 { seven: 127, one: 1 };
    let bytes1 = test1.to_be_bytes();
    assert_eq!(bytes1.len(), 1);
    let (parsed1, _) = DivCeilTest1::try_from_be_bytes(&bytes1).unwrap();
    assert_eq!(parsed1.seven, 127);
    assert_eq!(parsed1.one, 1);

    let test2 = DivCeilTest2 {
        fifteen: 32767,
        one: 1,
    };
    let bytes2 = test2.to_be_bytes();
    assert_eq!(bytes2.len(), 2);
    let (parsed2, _) = DivCeilTest2::try_from_be_bytes(&bytes2).unwrap();
    assert_eq!(parsed2.fifteen, 32767);
    assert_eq!(parsed2.one, 1);
}

#[test]
fn test_array_size_multiplication() {
    // Tests that array size calculation uses * not + or /

    #[derive(BeBytes, Debug, PartialEq)]
    struct ArrayMultTest1 {
        arr: [u8; 7], // 7 * 1 = 7 bytes
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct ArrayMultTest2 {
        arr: [u8; 13], // 13 * 1 = 13 bytes
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct ArrayMultTest3 {
        arr: [u8; 25], // 25 * 1 = 25 bytes
    }

    assert_eq!(ArrayMultTest1::field_size(), 7);
    assert_eq!(ArrayMultTest2::field_size(), 13);
    assert_eq!(ArrayMultTest3::field_size(), 25);

    // Verify the calculations work correctly
    let test1 = ArrayMultTest1 { arr: [0xAA; 7] };
    assert_eq!(test1.to_be_bytes().len(), 7);

    let test2 = ArrayMultTest2 { arr: [0xBB; 13] };
    assert_eq!(test2.to_be_bytes().len(), 13);

    let test3 = ArrayMultTest3 { arr: [0xCC; 25] };
    assert_eq!(test3.to_be_bytes().len(), 25);

    // Also test with multiple fields to ensure addition works
    #[derive(BeBytes, Debug, PartialEq)]
    struct MultiFieldArray {
        prefix: u16,
        arr: [u8; 10],
        suffix: u32,
    }

    // 2 + 10 + 4 = 16 bytes
    assert_eq!(MultiFieldArray::field_size(), 16);
}

#[test]
fn test_bit_position_addition() {
    // Tests that bit positions use += not *= or -=

    #[derive(BeBytes, Debug, PartialEq)]
    struct BitAddTest {
        #[bits(1)]
        a: u8,
        #[bits(2)]
        b: u8,
        #[bits(3)]
        c: u8,
        #[bits(4)]
        d: u8,
        #[bits(5)]
        e: u8,
        #[bits(6)]
        f: u8,
        #[bits(7)]
        g: u8,
        #[bits(4)]
        h: u8, // Total: 1+2+3+4+5+6+7+4 = 32 bits = 4 bytes
    }

    // If += is replaced with *= or -=, the size calculation will be wrong
    assert_eq!(BitAddTest::field_size(), 4);

    let test = BitAddTest {
        a: 1,
        b: 3,
        c: 7,
        d: 15,
        e: 31,
        f: 63,
        g: 127,
        h: 15,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 4);

    let (parsed, _) = BitAddTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.a, 1);
    assert_eq!(parsed.b, 3);
    assert_eq!(parsed.c, 7);
    assert_eq!(parsed.d, 15);
    assert_eq!(parsed.e, 31);
    assert_eq!(parsed.f, 63);
    assert_eq!(parsed.g, 127);
    assert_eq!(parsed.h, 15);
}

#[test]
fn test_byte_boundary_modulo() {
    // Tests that byte boundary checking uses % not / or +

    #[derive(BeBytes, Debug, PartialEq)]
    struct ModuloTest1 {
        #[bits(8)]
        exact: u8,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct ModuloTest2 {
        #[bits(16)]
        exact: u16,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct ModuloTest3 {
        #[bits(24)]
        exact: u32,
    }

    // These all divide evenly by 8, so % 8 == 0
    assert_eq!(ModuloTest1::field_size(), 1);
    assert_eq!(ModuloTest2::field_size(), 2);
    assert_eq!(ModuloTest3::field_size(), 3);
}

#[test]
fn test_bit_shift_direction() {
    // Tests that bit shifts use << not >>

    #[derive(BeBytes, Debug, PartialEq)]
    struct ShiftTest {
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

    // Test powers of 2 which rely on correct shift direction
    let test = ShiftTest {
        bit0: 1, // Should contribute 1 << 7 = 128
        bit1: 1, // Should contribute 1 << 6 = 64
        bit2: 1, // Should contribute 1 << 5 = 32
        bit3: 1, // Should contribute 1 << 4 = 16
        bit4: 1, // Should contribute 1 << 3 = 8
        bit5: 1, // Should contribute 1 << 2 = 4
        bit6: 1, // Should contribute 1 << 1 = 2
        bit7: 1, // Should contribute 1 << 0 = 1
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes[0], 255); // All bits set

    // Test specific bit patterns
    let test2 = ShiftTest {
        bit0: 1, // 128
        bit1: 0,
        bit2: 1, // 32
        bit3: 0,
        bit4: 1, // 8
        bit5: 0,
        bit6: 1, // 2
        bit7: 0,
    };

    let bytes2 = test2.to_be_bytes();
    assert_eq!(bytes2[0], 128 + 32 + 8 + 2); // 170 = 0xAA
}

#[test]
fn test_subtraction_operations() {
    // Test operations that use subtraction

    #[derive(BeBytes, Debug, PartialEq)]
    struct SubTest {
        #[bits(5)]
        five: u8,
        #[bits(3)]
        three: u8,
    }

    // Bit limit check uses (1 << bits) - 1
    let test = SubTest {
        five: 31, // Max 5-bit value: (1 << 5) - 1 = 31
        three: 7, // Max 3-bit value: (1 << 3) - 1 = 7
    };

    let bytes = test.to_be_bytes();
    let (parsed, _) = SubTest::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(parsed.five, 31);
    assert_eq!(parsed.three, 7);

    // Values should be masked to their bit sizes
    let test2 = SubTest {
        five: 63,  // Should be masked to 31
        three: 15, // Should be masked to 7
    };

    let bytes2 = test2.to_be_bytes();
    let (parsed2, _) = SubTest::try_from_be_bytes(&bytes2).unwrap();

    assert_eq!(parsed2.five, 31);
    assert_eq!(parsed2.three, 7);
}

#[test]
fn test_primitive_size_arithmetic() {
    // Test that primitive sizes are calculated correctly

    #[derive(BeBytes, Debug, PartialEq)]
    struct MixedPrimitives {
        a: u8,   // 1 byte
        b: u16,  // 2 bytes
        c: u32,  // 4 bytes
        d: u64,  // 8 bytes
        e: u128, // 16 bytes
    }

    // Total should be 1 + 2 + 4 + 8 + 16 = 31 bytes
    assert_eq!(MixedPrimitives::field_size(), 31);

    let test = MixedPrimitives {
        a: 0xFF,
        b: 0xFFFF,
        c: 0xFFFFFFFF,
        d: 0xFFFFFFFFFFFFFFFF,
        e: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 31);
}

#[test]
fn test_enum_discriminant_arithmetic() {
    // Test enum discriminant calculations

    #[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
    enum PowerOfTwo {
        One = 1,
        Two = 2,
        Four = 4,
        Eight = 8,
        Sixteen = 16,
        ThirtyTwo = 32,
        SixtyFour = 64,
        OneTwentyEight = 128,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct EnumArithTest {
        val: PowerOfTwo,
    }

    // Test all values to ensure discriminant handling is correct
    let values = [
        PowerOfTwo::One,
        PowerOfTwo::Two,
        PowerOfTwo::Four,
        PowerOfTwo::Eight,
        PowerOfTwo::Sixteen,
        PowerOfTwo::ThirtyTwo,
        PowerOfTwo::SixtyFour,
        PowerOfTwo::OneTwentyEight,
    ];

    for &val in &values {
        let test = EnumArithTest { val };
        let bytes = test.to_be_bytes();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], val as u8);

        let (parsed, _) = EnumArithTest::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.val as u8, val as u8);
    }
}
