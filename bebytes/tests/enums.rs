//! Enum functionality tests for BeBytes
//! 
//! This module tests:
//! - Basic enum serialization
//! - Auto-sized enum fields with #[bits()]
//! - Flag enums with bitwise operations
//! - Enum bit packing

use bebytes::BeBytes;

mod basic_enums {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
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
        ].iter().enumerate() {
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
    #[repr(u8)]
    enum TwoBitEnum {
        A = 0,
        B = 1,
        C = 2,
        D = 3,
    }

    #[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
    #[repr(u8)]
    enum ThreeBitEnum {
        V0 = 0,
        V1 = 1,
        V2 = 2,
        V3 = 3,
        V4 = 4,
    }

    #[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
    #[repr(u8)]
    enum FiveBitEnum {
        V0 = 0,
        V1 = 1,
        V16 = 16,
        V31 = 31,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct AutoSizedPacket {
        #[bits(4)]
        prefix: u8,
        #[bits()] // Auto-sized to 2 bits
        two_bit: TwoBitEnum,
        #[bits()] // Auto-sized to 3 bits
        three_bit: ThreeBitEnum,
        #[bits()] // Auto-sized to 5 bits
        five_bit: FiveBitEnum,
        #[bits(7)]
        suffix: u8,
    }

    #[test]
    fn test_auto_sized_enum_bits() {
        assert_eq!(TwoBitEnum::__BEBYTES_MIN_BITS, 2);
        assert_eq!(ThreeBitEnum::__BEBYTES_MIN_BITS, 3);
        assert_eq!(FiveBitEnum::__BEBYTES_MIN_BITS, 5);

        let packet = AutoSizedPacket {
            prefix: 0xF,
            two_bit: TwoBitEnum::C,
            three_bit: ThreeBitEnum::V4,
            five_bit: FiveBitEnum::V16,
            suffix: 0x55,
        };

        let bytes = packet.to_be_bytes();
        assert_eq!(bytes.len(), 3); // 4+2+3+5+7 = 21 bits = 3 bytes (rounded up)

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
            #[bits()]
            auto_enum: TwoBitEnum,
            #[bits(3)]
            more_manual: u8,
        }

        let packet = MixedPacket {
            manual_bits: 7,
            auto_enum: TwoBitEnum::B,
            more_manual: 5,
        };

        let bytes = packet.to_be_bytes();
        let (decoded, _) = MixedPacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, packet);
    }
}

mod flag_enums {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
    #[bebytes(flags)]
    enum StatusFlags {
        None = 0,
        Ready = 1,
        Busy = 2,
        ErrorStatus = 4,
        Complete = 8,
    }

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
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
            status: u8, // StatusFlags as u8
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
        assert_eq!(bytes[2], 9);  // Ready(1) | Complete(8) = 9
        assert_eq!(bytes[3], 7);  // Read(1) | Write(2) | Execute(4) = 7

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
}

mod enum_bit_packing {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
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
        #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
        #[repr(u8)]
        enum NonContiguous {
            First = 0,
            Second = 5,
            Third = 10,
            Fourth = 15,
        }

        // Should need 4 bits to represent value 15
        assert_eq!(NonContiguous::__BEBYTES_MIN_BITS, 4);

        #[derive(BeBytes, Debug, PartialEq)]
        struct NonContiguousPacket {
            #[bits(4)]
            prefix: u8,
            #[bits()]
            value: NonContiguous,
        }

        let packet = NonContiguousPacket {
            prefix: 0xA,
            value: NonContiguous::Third,
        };

        let bytes = packet.to_be_bytes();
        let (decoded, _) = NonContiguousPacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded.value, NonContiguous::Third);
    }
}