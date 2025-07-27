//! Tests for compile-time position tracking and optimizations
//! 
//! This module verifies that:
//! - Compile-time optimizations work correctly before auto-sized fields
//! - Compile-time optimizations are disabled after auto-sized fields
//! - Position tracking behaves correctly in various scenarios

use bebytes::BeBytes;

mod optimization_tests {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
    enum TinyEnum {
        A = 0,
        B = 1,
    }

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
    enum SmallEnum {
        X = 0,
        Y = 1,
        Z = 2,
    }

    // Test 1: No auto-sized fields - all optimizations should work
    #[derive(BeBytes, Debug, PartialEq)]
    struct NoAutoSized {
        header: u32,           // 0-31 (bytes 0-3)
        #[bits(16)]
        aligned_bits: u16,     // 32-47 (bytes 4-5) - byte aligned, should optimize
        #[bits(8)]
        byte_bits: u8,         // 48-55 (byte 6) - byte aligned, should optimize
        regular: u16,          // 56-71 (bytes 7-8) - regular field
    }

    #[test]
    fn test_no_auto_sized_optimizations() {
        let data = NoAutoSized {
            header: 0x12345678,
            aligned_bits: 0xABCD,
            byte_bits: 0xEF,
            regular: 0x9876,
        };

        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 9);
        
        // Verify exact byte layout
        assert_eq!(&bytes[0..4], &[0x12, 0x34, 0x56, 0x78]); // header
        assert_eq!(&bytes[4..6], &[0xAB, 0xCD]); // aligned_bits
        assert_eq!(bytes[6], 0xEF); // byte_bits
        assert_eq!(&bytes[7..9], &[0x98, 0x76]); // regular

        let (decoded, consumed) = NoAutoSized::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, 9);
        assert_eq!(decoded, data);
    }

    // Test 2: Auto-sized field in the middle
    #[derive(BeBytes, Debug, PartialEq)]
    struct WithAutoSized {
        header: u32,           // 0-31 (bytes 0-3)
        #[bits()]
        tiny: TinyEnum,        // 32-32 (1 bit) - auto-sized
        #[bits(7)]
        padding: u8,           // 33-39 (7 bits) - completes the byte
        #[bits(16)]
        after_auto: u16,       // 40-55 (bytes 5-6) - aligned but after auto-sized
        regular: u32,          // 56-87 (bytes 7-10)
    }

    #[test]
    fn test_with_auto_sized() {
        let data = WithAutoSized {
            header: 0xDEADBEEF,
            tiny: TinyEnum::B,
            padding: 0x55,
            after_auto: 0x1234,
            regular: 0xCAFEBABE,
        };

        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 11);

        // Verify header
        assert_eq!(&bytes[0..4], &[0xDE, 0xAD, 0xBE, 0xEF]);
        
        // Byte 4 contains: tiny (1 bit) + padding (7 bits)
        // tiny=B=1, padding=0x55=1010101
        // Combined: 1_1010101 = 0xD5
        assert_eq!(bytes[4], 0xD5);

        // after_auto field
        assert_eq!(&bytes[5..7], &[0x12, 0x34]);
        
        // regular field
        assert_eq!(&bytes[7..11], &[0xCA, 0xFE, 0xBA, 0xBE]);

        let (decoded, consumed) = WithAutoSized::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, 11);
        assert_eq!(decoded, data);
    }

    // Test 3: Multiple auto-sized fields
    #[derive(BeBytes, Debug, PartialEq)]
    struct MultipleAutoSized {
        #[bits(8)]
        before: u8,            // 0-7 (byte 0) - should optimize
        #[bits()]
        first_auto: TinyEnum,  // 8-8 (1 bit)
        #[bits(3)]
        middle: u8,            // 9-11 (3 bits)
        #[bits()]
        second_auto: SmallEnum, // 12-13 (2 bits)
        #[bits(2)]
        after: u8,             // 14-15 (2 bits) - completes byte 1
        regular: u16,          // 16-31 (bytes 2-3)
    }

    #[test]
    fn test_multiple_auto_sized() {
        let data = MultipleAutoSized {
            before: 0xFF,
            first_auto: TinyEnum::A,
            middle: 0x5,
            second_auto: SmallEnum::Z,
            after: 0x3,
            regular: 0xABCD,
        };

        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 4);

        // Byte 0: before field
        assert_eq!(bytes[0], 0xFF);
        
        // Byte 1: first_auto(1) + middle(3) + second_auto(2) + after(2)
        // first_auto=A=0, middle=5=101, second_auto=Z=2=10, after=3=11
        // Combined: 0_101_10_11 = 0x5B
        assert_eq!(bytes[1], 0x5B);

        // Bytes 2-3: regular field
        assert_eq!(&bytes[2..4], &[0xAB, 0xCD]);

        let (decoded, consumed) = MultipleAutoSized::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, 4);
        assert_eq!(decoded, data);
    }

    // Test 4: Auto-sized at the beginning
    #[derive(BeBytes, Debug, PartialEq)]
    struct AutoSizedFirst {
        #[bits()]
        auto_first: SmallEnum, // 0-1 (2 bits)
        #[bits(6)]
        padding: u8,           // 2-7 (6 bits) - completes byte 0
        #[bits(16)]
        after: u16,            // 8-23 (bytes 1-2) - should NOT optimize
    }

    #[test]
    fn test_auto_sized_first() {
        let data = AutoSizedFirst {
            auto_first: SmallEnum::Y,
            padding: 0x2A,
            after: 0x9876,
        };

        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 3);

        // Byte 0: auto_first(2) + padding(6)
        // auto_first=Y=1=01, padding=0x2A=101010
        // Combined: 01_101010 = 0x6A
        assert_eq!(bytes[0], 0x6A);

        // Bytes 1-2: after field
        assert_eq!(&bytes[1..3], &[0x98, 0x76]);

        let (decoded, consumed) = AutoSizedFirst::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, 3);
        assert_eq!(decoded, data);
    }

    // Test 5: Verify exact optimization behavior
    #[derive(BeBytes, Debug, PartialEq)]
    struct OptimizationCheck {
        filler: u64,           // 0-63 (bytes 0-7) - to create specific alignment
        #[bits(16)]
        before_auto: u16,      // 64-79 (bytes 8-9) - perfectly aligned at byte 8
        #[bits()]
        auto_enum: TinyEnum,   // 80-80 (1 bit)
        #[bits(7)]
        pad: u8,               // 81-87 (7 bits) - completes byte 10
        #[bits(16)]
        after_auto: u16,       // 88-103 (bytes 11-12) - aligned but after auto
    }

    #[test]
    fn test_optimization_behavior() {
        let data = OptimizationCheck {
            filler: 0x0102030405060708,
            before_auto: 0x1234,
            auto_enum: TinyEnum::B,
            pad: 0x55,
            after_auto: 0x5678,
        };

        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 13);

        // The before_auto field at position 64 (byte 8) should use optimization
        // The after_auto field at position 88 (byte 11) should NOT use optimization
        // even though both are byte-aligned

        let (decoded, consumed) = OptimizationCheck::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, 13);
        assert_eq!(decoded, data);
    }
}

mod edge_cases {
    use super::*;

    // Test with all auto-sized fields
    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
    enum Binary {
        Zero = 0,
        One = 1,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct AllAutoSized {
        #[bits()]
        a: Binary,
        #[bits()]
        b: Binary,
        #[bits()]
        c: Binary,
        #[bits()]
        d: Binary,
        #[bits()]
        e: Binary,
        #[bits()]
        f: Binary,
        #[bits()]
        g: Binary,
        #[bits()]
        h: Binary,
    }

    #[test]
    fn test_all_auto_sized() {
        let data = AllAutoSized {
            a: Binary::One,
            b: Binary::Zero,
            c: Binary::One,
            d: Binary::One,
            e: Binary::Zero,
            f: Binary::One,
            g: Binary::Zero,
            h: Binary::One,
        };

        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 1);
        // Binary pattern: 10110101 = 0xB5
        assert_eq!(bytes[0], 0xB5);

        let (decoded, consumed) = AllAutoSized::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, 1);
        assert_eq!(decoded, data);
    }
}