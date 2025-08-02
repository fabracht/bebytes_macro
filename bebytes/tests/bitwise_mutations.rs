//! Tests specifically targeting bitwise operator mutations
//! These ensure that &, |, ^, <<, >> are used correctly

use bebytes::BeBytes;

#[test]
fn test_bit_and_operations() {
    // Test operations that use & not ^ or |

    #[derive(BeBytes, Debug, PartialEq)]
    struct BitAndTest {
        #[bits(4)]
        masked: u8,
        #[bits(4)]
        other: u8,
    }

    // Test masking operations
    let test = BitAndTest {
        masked: 0xFF, // Should be masked to 0xF
        other: 0xFF,  // Should be masked to 0xF
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes[0], 0xFF); // Both nibbles set

    let (parsed, _) = BitAndTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.masked, 0xF);
    assert_eq!(parsed.other, 0xF);
}

#[test]
fn test_bit_or_merging() {
    // Test that bit merging uses |= not ^=

    #[derive(BeBytes, Debug, PartialEq)]
    struct BitOrTest {
        #[bits(1)]
        bit7: u8,
        #[bits(1)]
        bit6: u8,
        #[bits(1)]
        bit5: u8,
        #[bits(1)]
        bit4: u8,
        #[bits(1)]
        bit3: u8,
        #[bits(1)]
        bit2: u8,
        #[bits(1)]
        bit1: u8,
        #[bits(1)]
        bit0: u8,
    }

    // Test pattern where XOR would give different results than OR
    let test = BitOrTest {
        bit7: 1,
        bit6: 1,
        bit5: 0,
        bit4: 0,
        bit3: 1,
        bit2: 1,
        bit1: 0,
        bit0: 0,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes[0], 0b11001100); // 0xCC

    // Multiple writes to same positions would fail with XOR
    let test2 = BitOrTest {
        bit7: 1,
        bit6: 0,
        bit5: 1,
        bit4: 0,
        bit3: 1,
        bit2: 0,
        bit1: 1,
        bit0: 0,
    };

    let bytes2 = test2.to_be_bytes();
    assert_eq!(bytes2[0], 0b10101010); // 0xAA
}

#[test]
fn test_shift_left_operations() {
    // Test that bit positioning uses << not >>

    #[derive(BeBytes, Debug, PartialEq)]
    struct ShiftLeftTest {
        #[bits(4)]
        high_nibble: u8,
        #[bits(4)]
        low_nibble: u8,
    }

    let test = ShiftLeftTest {
        high_nibble: 0xA,
        low_nibble: 0x5,
    };

    let bytes = test.to_be_bytes();
    // high_nibble should be shifted left by 4: 0xA << 4 = 0xA0
    // Combined with low_nibble: 0xA0 | 0x05 = 0xA5
    assert_eq!(bytes[0], 0xA5);

    let (parsed, _) = ShiftLeftTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.high_nibble, 0xA);
    assert_eq!(parsed.low_nibble, 0x5);
}

#[test]
fn test_shift_right_operations() {
    // Test parsing operations that use >> to extract bits

    #[derive(BeBytes, Debug, PartialEq)]
    struct ShiftRightTest {
        #[bits(3)]
        top_three: u8,
        #[bits(5)]
        bottom_five: u8,
    }

    let test = ShiftRightTest {
        top_three: 0b111,
        bottom_five: 0b11111,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes[0], 0xFF); // 111_11111

    // Parse back - this tests the >> operations
    let (parsed, _) = ShiftRightTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.top_three, 0b111);
    assert_eq!(parsed.bottom_five, 0b11111);
}

#[test]
fn test_complex_bit_patterns() {
    // Test complex bit patterns that would fail with wrong operators

    #[derive(BeBytes, Debug, PartialEq)]
    struct ComplexBitTest {
        #[bits(10)]
        ten_bits: u16,
        #[bits(6)]
        six_bits: u8,
        #[bits(20)]
        twenty_bits: u32,
        #[bits(12)]
        twelve_bits: u16,
    }

    let test = ComplexBitTest {
        ten_bits: 0x3FF,      // Max 10-bit value
        six_bits: 0x3F,       // Max 6-bit value
        twenty_bits: 0xFFFFF, // Max 20-bit value
        twelve_bits: 0xFFF,   // Max 12-bit value
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 6); // 48 bits total

    let (parsed, _) = ComplexBitTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.ten_bits, 0x3FF);
    assert_eq!(parsed.six_bits, 0x3F);
    assert_eq!(parsed.twenty_bits, 0xFFFFF);
    assert_eq!(parsed.twelve_bits, 0xFFF);
}

#[test]
fn test_enum_bit_counting() {
    // Test enum discriminant bit counting with >>= vs <<=

    #[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
    enum BitCountEnum {
        Zero = 0,
        One = 1,
        Two = 2,
        Three = 3,
        OneTwentyEight = 128,
        TwoFiftyFive = 255,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct EnumBitTest {
        value: BitCountEnum,
    }

    // Test various discriminants
    let values = [
        BitCountEnum::Zero,
        BitCountEnum::One,
        BitCountEnum::Two,
        BitCountEnum::Three,
        BitCountEnum::OneTwentyEight,
        BitCountEnum::TwoFiftyFive,
    ];

    for &value in &values {
        let test = EnumBitTest { value };
        let bytes = test.to_be_bytes();
        assert_eq!(bytes[0], value as u8);

        let (parsed, _) = EnumBitTest::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value as u8, value as u8);
    }
}

#[test]
fn test_bit_limit_masks() {
    // Test that bit limits use correct shift operations

    #[derive(BeBytes, Debug, PartialEq)]
    struct BitLimitTest {
        #[bits(1)]
        one: u8,
        #[bits(7)]
        seven: u8,
        #[bits(9)]
        nine: u16,
        #[bits(15)]
        fifteen: u16,
    }

    let test = BitLimitTest {
        one: 1,
        seven: 127,
        nine: 511,
        fifteen: 32767,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 4); // 32 bits total

    let (parsed, _) = BitLimitTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.one, 1);
    assert_eq!(parsed.seven, 127);
    assert_eq!(parsed.nine, 511);
    assert_eq!(parsed.fifteen, 32767);

    // Test maximum valid values
    let max_valid = BitLimitTest {
        one: 1,         // Max 1-bit: 2^1 - 1 = 1
        seven: 127,     // Max 7-bit: 2^7 - 1 = 127
        nine: 511,      // Max 9-bit: 2^9 - 1 = 511
        fifteen: 32767, // Max 15-bit: 2^15 - 1 = 32767
    };

    let max_bytes = max_valid.to_be_bytes();
    let (parsed_max, _) = BitLimitTest::try_from_be_bytes(&max_bytes).unwrap();

    assert_eq!(parsed_max.one, 1);
    assert_eq!(parsed_max.seven, 127);
    assert_eq!(parsed_max.nine, 511);
    assert_eq!(parsed_max.fifteen, 32767);
}

#[test]
fn test_cross_byte_bit_operations() {
    // Test bit operations that cross byte boundaries

    #[derive(BeBytes, Debug, PartialEq)]
    struct CrossByteTest {
        #[bits(12)]
        twelve: u16,
        #[bits(12)]
        twelve2: u16,
        #[bits(8)]
        eight: u8,
    }

    let test = CrossByteTest {
        twelve: 0xABC,
        twelve2: 0xDEF,
        eight: 0x12,
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes.len(), 4); // 32 bits

    // Verify the bit packing
    let (parsed, _) = CrossByteTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.twelve, 0xABC);
    assert_eq!(parsed.twelve2, 0xDEF);
    assert_eq!(parsed.eight, 0x12);
}

#[test]
fn test_single_bit_extraction() {
    // Test extracting single bits

    #[derive(BeBytes, Debug, PartialEq)]
    struct SingleBitExtract {
        #[bits(1)]
        msb: u8,
        #[bits(6)]
        middle: u8,
        #[bits(1)]
        lsb: u8,
    }

    // Test various patterns
    let patterns = [
        (1, 0b111111, 1), // All set
        (0, 0b000000, 0), // All clear
        (1, 0b101010, 0), // Alternating
        (0, 0b010101, 1), // Inverse alternating
    ];

    for (msb, middle, lsb) in patterns {
        let test = SingleBitExtract { msb, middle, lsb };
        let bytes = test.to_be_bytes();

        let expected = (msb << 7) | (middle << 1) | lsb;
        assert_eq!(bytes[0], expected);

        let (parsed, _) = SingleBitExtract::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.msb, msb);
        assert_eq!(parsed.middle, middle);
        assert_eq!(parsed.lsb, lsb);
    }
}
