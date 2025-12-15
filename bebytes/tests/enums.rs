//! Enum functionality tests for BeBytes
//!
//! This module tests:
//! - Basic enum serialization
//! - Bit fields with explicit sizes
//! - Flag enums with bitwise operations
//! - Bit packing optimization

use bebytes::BeBytes;

mod basic_enums {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    enum SimpleEnum {
        First = 0,
        Second = 1,
        Third = 2,
        Fourth = 3,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct EnumPacket {
        header: u8,
        status: SimpleEnum,
        footer: u16,
    }

    #[test]
    fn test_simple_enum_serialization() {
        let packet = EnumPacket {
            header: 0xAA,
            status: SimpleEnum::Third,
            footer: 0xBBCC,
        };

        let bytes = packet.to_be_bytes();
        assert_eq!(bytes.len(), 4);
        assert_eq!(bytes[0], 0xAA);
        assert_eq!(bytes[1], 2); // SimpleEnum::Third
        assert_eq!(bytes[2], 0xBB);
        assert_eq!(bytes[3], 0xCC);

        let (decoded, _) = EnumPacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn test_enum_all_variants() {
        for (i, variant) in [
            SimpleEnum::First,
            SimpleEnum::Second,
            SimpleEnum::Third,
            SimpleEnum::Fourth,
        ]
        .iter()
        .enumerate()
        {
            let packet = EnumPacket {
                header: 0xFF,
                status: *variant,
                footer: 0x1234,
            };

            let bytes = packet.to_be_bytes();
            assert_eq!(bytes[1], i as u8);

            let (decoded, _) = EnumPacket::try_from_be_bytes(&bytes).unwrap();
            assert_eq!(decoded.status, *variant);
        }
    }
}

mod auto_sized_enums {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
    enum TwoBitEnum {
        A = 0,
        B = 1,
        C = 2,
        D = 3,
    }

    #[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
    enum ThreeBitEnum {
        V0 = 0,
        V1 = 1,
        V2 = 2,
        V3 = 3,
        V4 = 4,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct AutoSizedPacket {
        #[bits(4)]
        prefix: u8,
        #[bits(2)]
        two_bit: u8,
        #[bits(2)]
        three_bit: u8,
        suffix: u8,
    }

    #[test]
    fn test_bit_packing() {
        let packet = AutoSizedPacket {
            prefix: 0xF,
            two_bit: 2,
            three_bit: 1,
            suffix: 0x55,
        };

        let bytes = packet.to_be_bytes();
        assert_eq!(bytes.len(), 2);

        let (decoded, _) = AutoSizedPacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn test_enum_try_from() {
        // Valid values
        assert_eq!(TwoBitEnum::try_from(0u8).unwrap(), TwoBitEnum::A);
        assert_eq!(TwoBitEnum::try_from(3u8).unwrap(), TwoBitEnum::D);

        // Invalid values
        assert!(TwoBitEnum::try_from(4u8).is_err());
        assert!(ThreeBitEnum::try_from(5u8).is_err());
    }

    #[test]
    fn test_mixed_auto_and_explicit_bits() {
        #[derive(BeBytes, Debug, PartialEq)]
        struct MixedPacket {
            #[bits(3)]
            manual_bits: u8,
            #[bits(2)]
            auto_enum: u8, // TwoBitEnum as u8
            #[bits(3)]
            more_manual: u8,
        }

        let packet = MixedPacket {
            manual_bits: 7,
            auto_enum: 1, // 1 = TwoBitEnum::B
            more_manual: 5,
        };

        let bytes = packet.to_be_bytes();
        let (decoded, _) = MixedPacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, packet);
    }
}

#[cfg(feature = "std")]
mod flag_enums {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[bebytes(flags)]
    enum StatusFlags {
        None = 0,
        Ready = 1,
        Busy = 2,
        ErrorStatus = 4,
        Complete = 8,
    }

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[bebytes(flags)]
    enum PermissionFlags {
        Read = 1,
        Write = 2,
        Execute = 4,
        Delete = 8,
        Admin = 16,
    }

    #[test]
    fn test_flag_enum_basic_operations() {
        // Test individual flags
        let ready = StatusFlags::Ready;
        assert!(ready.contains(StatusFlags::Ready));
        assert!(!ready.contains(StatusFlags::Busy));

        // Bitwise operations return u8
        let flags = StatusFlags::Ready | StatusFlags::Complete;
        // Since flags is u8, we need to check bits manually
        assert_ne!(flags & (StatusFlags::Ready as u8), 0);
        assert_ne!(flags & (StatusFlags::Complete as u8), 0);
        assert_eq!(flags & (StatusFlags::Busy as u8), 0);
        assert_eq!(flags & (StatusFlags::ErrorStatus as u8), 0);
    }

    #[test]
    fn test_flag_enum_bitwise_operations() {
        // OR returns u8
        let flags1 = StatusFlags::Ready | StatusFlags::Busy;
        assert_eq!(flags1, 3);

        // AND with u8
        let flags2 = flags1 & (StatusFlags::Ready as u8);
        assert_eq!(flags2, 1);

        // XOR returns u8
        let flags3 = StatusFlags::Ready ^ StatusFlags::Busy;
        assert_eq!(flags3, 3);

        // NOT returns u8
        let flags4 = !StatusFlags::None;
        assert_eq!(flags4, 255);
    }

    #[test]
    fn test_flag_enum_from_bits() {
        // Valid combination
        let bits = StatusFlags::from_bits(9).unwrap(); // Ready | Complete
        assert_eq!(bits, 9); // from_bits returns u8
        assert_ne!(bits & (StatusFlags::Ready as u8), 0);
        assert_ne!(bits & (StatusFlags::Complete as u8), 0);

        // Invalid bits
        assert!(StatusFlags::from_bits(16).is_none()); // Invalid bit set
    }

    #[test]
    fn test_flag_enum_in_struct() {
        #[derive(BeBytes, Debug, PartialEq)]
        struct FlagPacket {
            header: u16,
            status: u8,      // StatusFlags as u8
            permissions: u8, // PermissionFlags as u8
            data: u32,
        }

        let packet = FlagPacket {
            header: 0x1234,
            status: StatusFlags::Ready | StatusFlags::Complete,
            permissions: PermissionFlags::Read | PermissionFlags::Write | PermissionFlags::Execute,
            data: 0xDEADBEEF,
        };

        let bytes = packet.to_be_bytes();
        assert_eq!(bytes[2], 9); // Ready(1) | Complete(8) = 9
        assert_eq!(bytes[3], 7); // Read(1) | Write(2) | Execute(4) = 7

        let (decoded, _) = FlagPacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, packet);
        assert_ne!(decoded.status & (StatusFlags::Ready as u8), 0);
        assert_ne!(decoded.permissions & (PermissionFlags::Execute as u8), 0);
    }

    #[test]
    fn test_zero_flag_value() {
        let none = StatusFlags::None;
        assert_eq!(none as u8, 0);
        assert!(!none.contains(StatusFlags::Ready));

        // Zero should be valid in from_bits
        let from_zero = StatusFlags::from_bits(0).unwrap();
        assert_eq!(from_zero, 0u8); // from_bits returns u8
    }

    #[test]
    fn test_flag_decomposition() {
        // Test empty flags
        let empty_flags = StatusFlags::decompose(0);
        assert!(empty_flags.is_empty());

        // Test single flag
        let single_flags = StatusFlags::decompose(StatusFlags::Ready as u8);
        assert_eq!(single_flags.len(), 1);
        assert_eq!(single_flags[0], StatusFlags::Ready);

        // Test multiple flags
        let combined = StatusFlags::Ready as u8 | StatusFlags::Complete as u8;
        let multi_flags = StatusFlags::decompose(combined);
        assert_eq!(multi_flags.len(), 2);
        assert!(multi_flags.contains(&StatusFlags::Ready));
        assert!(multi_flags.contains(&StatusFlags::Complete));
        assert!(!multi_flags.contains(&StatusFlags::Busy));

        // Test all flags
        let all_flags_value = StatusFlags::Ready as u8
            | StatusFlags::Busy as u8
            | StatusFlags::Complete as u8
            | StatusFlags::ErrorStatus as u8;
        let all_flags = StatusFlags::decompose(all_flags_value);
        assert_eq!(all_flags.len(), 4);
        assert!(all_flags.contains(&StatusFlags::Ready));
        assert!(all_flags.contains(&StatusFlags::Busy));
        assert!(all_flags.contains(&StatusFlags::Complete));
        assert!(all_flags.contains(&StatusFlags::ErrorStatus));
    }

    #[test]
    fn test_flag_iter() {
        // Test iter_flags with multiple flags
        let combined = StatusFlags::Ready as u8 | StatusFlags::Complete as u8;
        let flag_iter: Vec<_> = StatusFlags::iter_flags(combined).collect();
        assert_eq!(flag_iter.len(), 2);
        assert!(flag_iter.contains(&StatusFlags::Ready));
        assert!(flag_iter.contains(&StatusFlags::Complete));

        // Test iter_flags with empty
        let empty_iter: Vec<_> = StatusFlags::iter_flags(0).collect();
        assert!(empty_iter.is_empty());

        // Test iter_flags with single flag
        let single_iter: Vec<_> = StatusFlags::iter_flags(StatusFlags::Busy as u8).collect();
        assert_eq!(single_iter.len(), 1);
        assert_eq!(single_iter[0], StatusFlags::Busy);
    }

    #[test]
    fn test_flag_decomposition_round_trip() {
        // Create combined flags
        let original = StatusFlags::Ready as u8 | StatusFlags::ErrorStatus as u8;

        // Decompose
        let decomposed = StatusFlags::decompose(original);

        // Reconstruct
        let reconstructed = decomposed.iter().fold(0u8, |acc, flag| acc | (*flag as u8));

        assert_eq!(original, reconstructed);
    }

    #[test]
    fn test_flag_decomposition_invalid_bits() {
        // Test with invalid bit combination (16 is not a valid flag)
        let invalid_flags = StatusFlags::decompose(16);
        assert!(invalid_flags.is_empty()); // Should return empty since 16 doesn't match any flag

        // Test with partially valid combination
        let partial = StatusFlags::Ready as u8 | 16; // Ready + invalid bit
        let partial_flags = StatusFlags::decompose(partial);
        assert_eq!(partial_flags.len(), 1);
        assert_eq!(partial_flags[0], StatusFlags::Ready);
    }
}

#[cfg(feature = "std")]
mod multibyte_flags {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[bebytes(flags)]
    enum U16Flags {
        None = 0,
        Bit0 = 1,
        Bit8 = 256,
        Bit15 = 32768,
    }

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[bebytes(flags(u32))]
    enum ExplicitU32Flags {
        None = 0,
        Low = 1,
        Mid = 65536,
        High = 0x8000_0000,
    }

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[bebytes(flags)]
    enum AutoDetectU32 {
        A = 1,
        B = 0x1_0000,
    }

    #[test]
    fn test_u16_flags_basic() {
        assert_eq!(U16Flags::field_size(), 2);

        let flag = U16Flags::Bit8;
        assert!(flag.contains(U16Flags::Bit8));
        assert!(!flag.contains(U16Flags::Bit0));

        let combined: u16 = U16Flags::Bit0 | U16Flags::Bit8;
        assert_eq!(combined, 257);
    }

    #[test]
    fn test_u16_flags_serialization() {
        let bytes = U16Flags::Bit8.to_be_bytes();
        assert_eq!(bytes, vec![0x01, 0x00]);

        let bytes_le = U16Flags::Bit8.to_le_bytes();
        assert_eq!(bytes_le, vec![0x00, 0x01]);

        let (parsed, consumed) = U16Flags::try_from_be_bytes(&[0x01, 0x00]).unwrap();
        assert_eq!(parsed, U16Flags::Bit8);
        assert_eq!(consumed, 2);
    }

    #[test]
    fn test_u16_flags_bitwise_ops() {
        let combined = U16Flags::Bit0 | U16Flags::Bit15;
        assert_eq!(combined, 32769);

        let masked = combined & U16Flags::Bit0;
        assert_eq!(masked, 1);

        let toggled = U16Flags::Bit0 ^ U16Flags::Bit0;
        assert_eq!(toggled, 0);

        let inverted = !U16Flags::None;
        assert_eq!(inverted, 0xFFFF);
    }

    #[test]
    fn test_u16_flags_from_bits() {
        let valid = U16Flags::from_bits(257);
        assert_eq!(valid, Some(257));

        let invalid = U16Flags::from_bits(2);
        assert!(invalid.is_none());
    }

    #[test]
    fn test_u16_flags_decompose() {
        let combined = U16Flags::Bit0 | U16Flags::Bit8 | U16Flags::Bit15;
        let decomposed = U16Flags::decompose(combined);
        assert_eq!(decomposed.len(), 3);
        assert!(decomposed.contains(&U16Flags::Bit0));
        assert!(decomposed.contains(&U16Flags::Bit8));
        assert!(decomposed.contains(&U16Flags::Bit15));
    }

    #[test]
    fn test_explicit_u32_flags() {
        assert_eq!(ExplicitU32Flags::field_size(), 4);

        let bytes = ExplicitU32Flags::High.to_be_bytes();
        assert_eq!(bytes, vec![0x80, 0x00, 0x00, 0x00]);

        let combined: u32 = ExplicitU32Flags::Low | ExplicitU32Flags::High;
        assert_eq!(combined, 0x8000_0001);
    }

    #[test]
    fn test_explicit_u32_flags_round_trip() {
        let bytes = ExplicitU32Flags::Mid.to_be_bytes();
        let (parsed, _) = ExplicitU32Flags::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed, ExplicitU32Flags::Mid);
    }

    #[test]
    fn test_auto_detect_u32() {
        assert_eq!(AutoDetectU32::field_size(), 4);

        let combined: u32 = AutoDetectU32::A | AutoDetectU32::B;
        assert_eq!(combined, 0x1_0001);
    }

    #[test]
    fn test_u32_flags_in_struct() {
        #[derive(BeBytes, Debug, PartialEq)]
        struct FlagPacket32 {
            header: u8,
            flags: u32,
            trailer: u8,
        }

        let packet = FlagPacket32 {
            header: 0xAA,
            flags: ExplicitU32Flags::Low | ExplicitU32Flags::High,
            trailer: 0xBB,
        };

        let bytes = packet.to_be_bytes();
        assert_eq!(bytes.len(), 6);
        assert_eq!(bytes[0], 0xAA);
        assert_eq!(&bytes[1..5], &[0x80, 0x00, 0x00, 0x01]);
        assert_eq!(bytes[5], 0xBB);

        let (decoded, _) = FlagPacket32::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn test_try_from_for_multibyte() {
        let val: u16 = 256;
        let flag = U16Flags::try_from(val).unwrap();
        assert_eq!(flag, U16Flags::Bit8);

        let invalid = U16Flags::try_from(2u16);
        assert!(invalid.is_err());
    }

    #[test]
    fn test_u16_flags_iter() {
        let combined: u16 = U16Flags::Bit0 | U16Flags::Bit15;
        let flags: Vec<_> = U16Flags::iter_flags(combined).collect();
        assert_eq!(flags.len(), 2);
        assert!(flags.contains(&U16Flags::Bit0));
        assert!(flags.contains(&U16Flags::Bit15));
    }

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[bebytes(flags)]
    enum U64Flags {
        None = 0,
        Bit0 = 1,
        Bit32 = 0x1_0000_0000,
        Bit62 = 0x4000_0000_0000_0000,
    }

    #[test]
    fn test_u64_flags_basic() {
        assert_eq!(U64Flags::field_size(), 8);

        let flag = U64Flags::Bit32;
        assert!(flag.contains(U64Flags::Bit32));
        assert!(!flag.contains(U64Flags::Bit0));

        let combined: u64 = U64Flags::Bit0 | U64Flags::Bit62;
        assert_eq!(combined, 0x4000_0000_0000_0001);
    }

    #[test]
    fn test_u64_flags_serialization() {
        let bytes = U64Flags::Bit32.to_be_bytes();
        assert_eq!(bytes, vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]);

        let bytes_le = U64Flags::Bit32.to_le_bytes();
        assert_eq!(
            bytes_le,
            vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]
        );

        let (parsed, consumed) = U64Flags::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed, U64Flags::Bit32);
        assert_eq!(consumed, 8);
    }

    #[test]
    fn test_u64_flags_bitwise_ops() {
        let combined = U64Flags::Bit0 | U64Flags::Bit62;
        assert_eq!(combined, 0x4000_0000_0000_0001);

        let masked = combined & U64Flags::Bit0;
        assert_eq!(masked, 1);

        let toggled = U64Flags::Bit0 ^ U64Flags::Bit0;
        assert_eq!(toggled, 0);
    }

    #[test]
    fn test_u64_flags_decompose() {
        let combined = U64Flags::Bit0 | U64Flags::Bit32 | U64Flags::Bit62;
        let decomposed = U64Flags::decompose(combined);
        assert_eq!(decomposed.len(), 3);
        assert!(decomposed.contains(&U64Flags::Bit0));
        assert!(decomposed.contains(&U64Flags::Bit32));
        assert!(decomposed.contains(&U64Flags::Bit62));
    }

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[bebytes(flags(u128))]
    enum U128Flags {
        None = 0,
        Bit0 = 1,
        Bit16 = 0x1_0000,
        Bit32 = 0x1_0000_0000,
    }

    #[test]
    fn test_u128_flags_basic() {
        assert_eq!(U128Flags::field_size(), 16);

        let flag = U128Flags::Bit32;
        assert!(flag.contains(U128Flags::Bit32));
        assert!(!flag.contains(U128Flags::Bit0));

        let combined: u128 = U128Flags::Bit0 | U128Flags::Bit32;
        assert_eq!(combined, 0x1_0000_0001);
    }

    #[test]
    fn test_u128_flags_serialization() {
        let bytes = U128Flags::Bit32.to_be_bytes();
        assert_eq!(bytes.len(), 16);
        assert_eq!(bytes[11], 0x01);

        let (parsed, consumed) = U128Flags::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed, U128Flags::Bit32);
        assert_eq!(consumed, 16);
    }

    #[test]
    fn test_u128_flags_decompose() {
        let combined = U128Flags::Bit0 | U128Flags::Bit16;
        let decomposed = U128Flags::decompose(combined);
        assert_eq!(decomposed.len(), 2);
        assert!(decomposed.contains(&U128Flags::Bit0));
        assert!(decomposed.contains(&U128Flags::Bit16));
    }

    #[test]
    fn test_invalid_discriminant_large_u16() {
        let invalid = U16Flags::try_from(2u16);
        match invalid {
            Err(bebytes::BeBytesError::InvalidDiscriminantLarge { value, type_name }) => {
                assert_eq!(value, 2);
                assert_eq!(type_name, "U16Flags");
            }
            _ => panic!("Expected InvalidDiscriminantLarge error"),
        }
    }

    #[test]
    fn test_invalid_discriminant_large_u32() {
        let invalid = ExplicitU32Flags::try_from(3u32);
        match invalid {
            Err(bebytes::BeBytesError::InvalidDiscriminantLarge { value, type_name }) => {
                assert_eq!(value, 3);
                assert_eq!(type_name, "ExplicitU32Flags");
            }
            _ => panic!("Expected InvalidDiscriminantLarge error"),
        }
    }

    #[test]
    fn test_invalid_discriminant_large_u64() {
        let invalid = U64Flags::try_from(5u64);
        match invalid {
            Err(bebytes::BeBytesError::InvalidDiscriminantLarge { value, type_name }) => {
                assert_eq!(value, 5);
                assert_eq!(type_name, "U64Flags");
            }
            _ => panic!("Expected InvalidDiscriminantLarge error"),
        }
    }

    #[test]
    fn test_invalid_discriminant_large_from_bytes() {
        let bytes = [0x00, 0x02];
        let result = U16Flags::try_from_be_bytes(&bytes);
        match result {
            Err(bebytes::BeBytesError::InvalidDiscriminantLarge { value, type_name }) => {
                assert_eq!(value, 2);
                assert_eq!(type_name, "U16Flags");
            }
            _ => panic!("Expected InvalidDiscriminantLarge error"),
        }
    }

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[bebytes(flags)]
    enum MaxU8Flags {
        Bit7 = 128,
    }

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[bebytes(flags)]
    enum MinU16Flags {
        Bit8 = 256,
    }

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[bebytes(flags)]
    enum MaxU16Flags {
        Bit15 = 32768,
    }

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[bebytes(flags)]
    enum MinU32Flags {
        Bit16 = 65536,
    }

    #[test]
    fn test_auto_detect_boundary_u8_max() {
        assert_eq!(MaxU8Flags::field_size(), 1);
        let bytes = MaxU8Flags::Bit7.to_be_bytes();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 128);
    }

    #[test]
    fn test_auto_detect_boundary_u16_min() {
        assert_eq!(MinU16Flags::field_size(), 2);
        let bytes = MinU16Flags::Bit8.to_be_bytes();
        assert_eq!(bytes.len(), 2);
        assert_eq!(bytes, vec![0x01, 0x00]);
    }

    #[test]
    fn test_auto_detect_boundary_u16_max() {
        assert_eq!(MaxU16Flags::field_size(), 2);
        let bytes = MaxU16Flags::Bit15.to_be_bytes();
        assert_eq!(bytes.len(), 2);
        assert_eq!(bytes, vec![0x80, 0x00]);
    }

    #[test]
    fn test_auto_detect_boundary_u32_min() {
        assert_eq!(MinU32Flags::field_size(), 4);
        let bytes = MinU32Flags::Bit16.to_be_bytes();
        assert_eq!(bytes.len(), 4);
        assert_eq!(bytes, vec![0x00, 0x01, 0x00, 0x00]);
    }

    #[test]
    fn test_explicit_type_forces_larger_size() {
        #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
        #[bebytes(flags(u32))]
        enum SmallButU32 {
            Flag1 = 1,
            Flag2 = 2,
        }

        assert_eq!(SmallButU32::field_size(), 4);
        let bytes = SmallButU32::Flag1.to_be_bytes();
        assert_eq!(bytes, vec![0x00, 0x00, 0x00, 0x01]);
    }
}

mod enum_bit_packing {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    enum PackedEnum {
        A = 0,
        B = 1,
        C = 2,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct BitPackedEnums {
        #[bits(2)]
        enum1: u8,
        #[bits(2)]
        enum2: u8,
        #[bits(2)]
        enum3: u8,
        #[bits(2)]
        enum4: u8,
    }

    #[test]
    fn test_enum_bit_packing() {
        let packet = BitPackedEnums {
            enum1: PackedEnum::A as u8,
            enum2: PackedEnum::B as u8,
            enum3: PackedEnum::C as u8,
            enum4: PackedEnum::A as u8,
        };

        let bytes = packet.to_be_bytes();
        assert_eq!(bytes.len(), 1);
        // 00 | 01 | 10 | 00 = 0b00011000 = 24
        assert_eq!(bytes[0], 0b00011000);

        let (decoded, _) = BitPackedEnums::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn test_non_contiguous_enum_values() {
        #[derive(BeBytes, Debug, PartialEq)]
        struct NonContiguousPacket {
            #[bits(4)]
            prefix: u8,
            #[bits(4)]
            value: u8,
        }

        let packet = NonContiguousPacket {
            prefix: 0xA,
            value: 10,
        };

        let bytes = packet.to_be_bytes();
        let (decoded, _) = NonContiguousPacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded.value, 10);
    }
}
