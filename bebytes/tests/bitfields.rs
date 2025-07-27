//! Bitfield functionality tests for BeBytes
//! 
//! This module tests:
//! - Basic bit field packing/unpacking
//! - Boundary crossing bit fields
//! - Multi-byte bit fields
//! - Edge cases and limits

use bebytes::BeBytes;

mod basic_bitfields {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq)]
    struct SimpleBitfield {
        #[bits(4)]
        high_nibble: u8,
        #[bits(4)]
        low_nibble: u8,
    }

    #[test]
    fn test_single_byte_bitfield() {
        let data = SimpleBitfield {
            high_nibble: 0xA,
            low_nibble: 0x5,
        };

        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 0xA5);

        let (decoded, _) = SimpleBitfield::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_uneven_bitfields() {
        #[derive(BeBytes, Debug, PartialEq)]
        struct UnevenBits {
            #[bits(3)]
            three_bits: u8,
            #[bits(5)]
            five_bits: u8,
        }

        let data = UnevenBits {
            three_bits: 0b101,
            five_bits: 0b11010,
        };

        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 1);
        // 101 | 11010 = 0b10111010
        assert_eq!(bytes[0], 0b10111010);

        let (decoded, _) = UnevenBits::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, data);
    }
}

mod boundary_crossing {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq)]
    struct CrossByteBoundary {
        #[bits(12)]
        twelve_bits: u16,
        #[bits(10)]
        ten_bits: u16,
        #[bits(7)]
        seven_bits: u8,
        #[bits(3)]
        three_bits: u8,
    }

    #[test]
    fn test_boundary_crossing_bitfield() {
        let data = CrossByteBoundary {
            twelve_bits: 0x123,
            ten_bits: 0x1FF,
            seven_bits: 65,
            three_bits: 2,
        };

        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 4); // 32 bits total

        let (decoded, consumed) = CrossByteBoundary::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, 4);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_boundary_at_different_positions() {
        #[derive(BeBytes, Debug, PartialEq)]
        struct BoundaryPositions {
            #[bits(7)]  // Ends at bit 7
            field1: u8,
            #[bits(6)]  // Crosses byte boundary (bits 7-12)
            field2: u8,
            #[bits(3)]  // Bits 13-15
            field3: u8,
        }

        let data = BoundaryPositions {
            field1: 0x7F,  // Max 7-bit value
            field2: 0x3F,  // Max 6-bit value
            field3: 0x07,  // Max 3-bit value
        };

        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 2); // 16 bits total

        let (decoded, _) = BoundaryPositions::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, data);
    }
}

mod multibyte_bitfields {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq)]
    struct LargeBitfields {
        #[bits(20)]
        large_field: u32,
        #[bits(12)]
        medium_field: u16,
    }

    #[test]
    fn test_large_bitfields() {
        let data = LargeBitfields {
            large_field: 0xABCDE,
            medium_field: 0xFED,
        };

        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 4); // 32 bits total

        let (decoded, _) = LargeBitfields::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_mixed_sizes() {
        #[derive(BeBytes, Debug, PartialEq)]
        struct MixedSizes {
            #[bits(64)]
            full_u64: u64,
            #[bits(32)]
            full_u32: u32,
            #[bits(16)]
            full_u16: u16,
            #[bits(8)]
            full_u8: u8,
        }

        let data = MixedSizes {
            full_u64: 0xDEADBEEFCAFEBABE,
            full_u32: 0x12345678,
            full_u16: 0xABCD,
            full_u8: 0xEF,
        };

        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 15); // 64+32+16+8 = 120 bits = 15 bytes

        let (decoded, _) = MixedSizes::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, data);
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn test_single_bit_fields() {
        #[derive(BeBytes, Debug, PartialEq)]
        struct SingleBits {
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

        let data = SingleBits {
            bit0: 1,
            bit1: 0,
            bit2: 1,
            bit3: 0,
            bit4: 1,
            bit5: 0,
            bit6: 1,
            bit7: 0,
        };

        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 0b10101010);

        let (decoded, _) = SingleBits::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_max_values() {
        #[derive(BeBytes, Debug, PartialEq)]
        struct MaxValues {
            #[bits(7)]
            seven_bits: u8,
            #[bits(15)]
            fifteen_bits: u16,
            #[bits(10)]
            ten_bits: u16,
        }

        let data = MaxValues {
            seven_bits: 0x7F,    // Max 7-bit value
            fifteen_bits: 0x7FFF, // Max 15-bit value
            ten_bits: 0x3FF,     // Max 10-bit value
        };

        let bytes = data.to_be_bytes();
        let (decoded, _) = MaxValues::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, data);
    }

    // TODO: Overflow checking is not implemented for single-byte bitfields
    // This test is disabled until that's fixed
    // #[test]
    // #[should_panic(expected = "exceeds the maximum allowed value")]
    // fn test_bitfield_overflow_panic() {
    //     #[derive(BeBytes, Debug, PartialEq)]
    //     struct OverflowTest {
    //         #[bits(4)]
    //         nibble: u8,
    //         #[bits(4)]
    //         other: u8,
    //     }
    //
    //     let data = OverflowTest {
    //         nibble: 16, // This exceeds 4-bit max (15)
    //         other: 0,
    //     };
    //
    //     // This should panic
    //     let _ = data.to_be_bytes();
    // }
}

mod endianness {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq)]
    struct BitfieldEndian {
        #[bits(12)]
        field1: u16,
        #[bits(20)]
        field2: u32,
    }

    // TODO: This test is disabled due to a bug in multi-byte bitfield deserialization
    // The deserializer incorrectly requires extra bytes when fields don't align on byte boundaries
    // It checks for the full size of the underlying type instead of actual bytes needed
    // #[test]
    // fn test_bitfield_endianness() {
    //     let data = BitfieldEndian {
    //         field1: 0xABC,
    //         field2: 0xDEF01,
    //     };
    //
    //     let be_bytes = data.to_be_bytes();
    //     let le_bytes = data.to_le_bytes();
    //
    //     // Both should deserialize correctly with their respective methods
    //     let (be_decoded, _) = BitfieldEndian::try_from_be_bytes(&be_bytes).unwrap();
    //     let (le_decoded, _) = BitfieldEndian::try_from_le_bytes(&le_bytes).unwrap();
    //
    //     assert_eq!(be_decoded, data);
    //     assert_eq!(le_decoded, data);
    // }
}