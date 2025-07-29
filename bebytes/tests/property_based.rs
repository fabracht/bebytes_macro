//! Property-based tests for BeBytes using proptest

use bebytes::BeBytes;
use proptest::prelude::*;

// Round-trip property tests for primitive types
proptest! {
    #[test]
    fn prop_u8_round_trip(value: u8) {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestU8 {
            field: u8,
        }

        let original = TestU8 { field: value };

        // Test big-endian
        let be_bytes = original.to_be_bytes();
        let (be_decoded, be_consumed) = TestU8::try_from_be_bytes(&be_bytes).unwrap();
        prop_assert_eq!(&original, &be_decoded);
        prop_assert_eq!(be_consumed, 1);

        // Test little-endian
        let le_bytes = original.to_le_bytes();
        let (le_decoded, le_consumed) = TestU8::try_from_le_bytes(&le_bytes).unwrap();
        prop_assert_eq!(&original, &le_decoded);
        prop_assert_eq!(le_consumed, 1);
    }

    #[test]
    fn prop_u16_round_trip(value: u16) {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestU16 {
            field: u16,
        }

        let original = TestU16 { field: value };

        // Test big-endian
        let be_bytes = original.to_be_bytes();
        let (be_decoded, be_consumed) = TestU16::try_from_be_bytes(&be_bytes).unwrap();
        prop_assert_eq!(&original, &be_decoded);
        prop_assert_eq!(be_consumed, 2);

        // Test little-endian
        let le_bytes = original.to_le_bytes();
        let (le_decoded, le_consumed) = TestU16::try_from_le_bytes(&le_bytes).unwrap();
        prop_assert_eq!(&original, &le_decoded);
        prop_assert_eq!(le_consumed, 2);
    }

    #[test]
    fn prop_u32_round_trip(value: u32) {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestU32 {
            field: u32,
        }

        let original = TestU32 { field: value };

        // Test round-trip
        let bytes = original.to_be_bytes();
        let (decoded, _) = TestU32::try_from_be_bytes(&bytes).unwrap();
        prop_assert_eq!(&original, &decoded);
    }

    #[test]
    fn prop_mixed_primitives_round_trip(a: u8, b: u16, c: u32, d: i32) {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestMixed {
            field_a: u8,
            field_b: u16,
            field_c: u32,
            field_d: i32,
        }

        let original = TestMixed {
            field_a: a,
            field_b: b,
            field_c: c,
            field_d: d,
        };

        // Test big-endian
        let be_bytes = original.to_be_bytes();
        let (be_decoded, _) = TestMixed::try_from_be_bytes(&be_bytes).unwrap();
        prop_assert_eq!(&original, &be_decoded);

        // Test little-endian
        let le_bytes = original.to_le_bytes();
        let (le_decoded, _) = TestMixed::try_from_le_bytes(&le_bytes).unwrap();
        prop_assert_eq!(&original, &le_decoded);
    }
}

// Bit field property tests
proptest! {
    #[test]
    fn prop_bit_field_bounds(value: u8) {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestBits {
            #[bits(3)]
            three_bits: u8,
            #[bits(5)]
            five_bits: u8,
        }

        // Mask values to fit in their bit fields
        let three_bits = value & 0b111;
        let five_bits = (value >> 3) & 0b11111;

        let original = TestBits {
            three_bits,
            five_bits,
        };

        let bytes = original.to_be_bytes();
        let (decoded, _) = TestBits::try_from_be_bytes(&bytes).unwrap();

        prop_assert_eq!(original.three_bits, decoded.three_bits);
        prop_assert_eq!(original.five_bits, decoded.five_bits);
    }

    #[test]
    fn prop_multi_byte_bit_fields(value: u16) {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestMultiByteBits {
            #[bits(4)]
            small: u8,
            #[bits(10)]
            medium: u16,
            #[bits(2)]
            tiny: u8,
        }

        let small = (value & 0xF) as u8;
        let medium = (value >> 4) & 0x3FF;
        let tiny = ((value >> 14) & 0x3) as u8;

        let original = TestMultiByteBits {
            small,
            medium,
            tiny,
        };

        let bytes = original.to_be_bytes();
        let (decoded, _) = TestMultiByteBits::try_from_be_bytes(&bytes).unwrap();

        prop_assert_eq!(&original, &decoded);
    }
}

// String property tests
proptest! {
    #[test]
    fn prop_fixed_string_round_trip(s in "[a-zA-Z0-9]{16}") {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestFixedString {
            #[With(size(16))]
            name: String,
        }

        let original = TestFixedString {
            name: s,
        };

        let bytes = original.to_be_bytes();
        let (decoded, _) = TestFixedString::try_from_be_bytes(&bytes).unwrap();

        prop_assert_eq!(&original, &decoded);
    }

    #[test]
    fn prop_variable_string_round_trip(s in prop::string::string_regex("[a-zA-Z0-9]{0,255}").unwrap()) {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestVarString {
            len: u8,
            #[FromField(len)]
            content: String,
        }

        let original = TestVarString {
            len: s.len() as u8,
            content: s,
        };

        let bytes = original.to_be_bytes();
        let (decoded, _) = TestVarString::try_from_be_bytes(&bytes).unwrap();

        prop_assert_eq!(&original, &decoded);
    }

    #[test]
    fn prop_unicode_string_round_trip(s in "\\PC{0,50}") {
        // Test with arbitrary unicode strings (up to 50 chars)
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestUnicodeString {
            len: u16,
            #[FromField(len)]
            content: String,
        }

        let original = TestUnicodeString {
            len: s.len() as u16,
            content: s,
        };

        let bytes = original.to_be_bytes();
        let (decoded, _) = TestUnicodeString::try_from_be_bytes(&bytes).unwrap();

        prop_assert_eq!(&original, &decoded);
    }
}

// Endianness consistency tests
proptest! {
    #[test]
    fn prop_endianness_consistency(value: u32) {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestEndian {
            field: u32,
        }

        let data = TestEndian { field: value };

        let be_bytes = data.to_be_bytes();
        let le_bytes = data.to_le_bytes();

        // Verify that BE and LE are byte-reversed
        prop_assert_eq!(be_bytes.len(), le_bytes.len());
        prop_assert_eq!(be_bytes.len(), 4);

        // The bytes should be reversed
        prop_assert_eq!(be_bytes[0], le_bytes[3]);
        prop_assert_eq!(be_bytes[1], le_bytes[2]);
        prop_assert_eq!(be_bytes[2], le_bytes[1]);
        prop_assert_eq!(be_bytes[3], le_bytes[0]);
    }

    #[test]
    fn prop_endianness_round_trip_consistency(value: u32) {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestEndian {
            field: u32,
        }

        let original = TestEndian { field: value };

        // Serialize with BE, deserialize with BE
        let be_bytes = original.to_be_bytes();
        let (be_decoded, _) = TestEndian::try_from_be_bytes(&be_bytes).unwrap();
        prop_assert_eq!(&original, &be_decoded);

        // Serialize with LE, deserialize with LE
        let le_bytes = original.to_le_bytes();
        let (le_decoded, _) = TestEndian::try_from_le_bytes(&le_bytes).unwrap();
        prop_assert_eq!(&original, &le_decoded);

        // Cross-endian should NOT work correctly (values should differ)
        if value != 0 && value != u32::from_be_bytes(value.to_le_bytes()) {
            let (cross_decoded, _) = TestEndian::try_from_be_bytes(&le_bytes).unwrap();
            prop_assert_ne!(original.field, cross_decoded.field);
        }
    }
}

// Size calculation tests
proptest! {
    #[test]
    fn prop_size_calculation_consistency(value: u32) {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestSize {
            field: u32,
        }

        let data = TestSize { field: value };

        let be_bytes = data.to_be_bytes();
        let le_bytes = data.to_le_bytes();

        // Size should match field_size()
        prop_assert_eq!(be_bytes.len(), TestSize::field_size());
        prop_assert_eq!(le_bytes.len(), TestSize::field_size());
        prop_assert_eq!(TestSize::field_size(), 4);
    }

    #[test]
    fn prop_complex_size_calculation(a: u8, b: u16, s in "[a-zA-Z0-9]{10}") {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestComplexSize {
            field_a: u8,
            #[bits(4)]
            bits_a: u8,
            #[bits(4)]
            bits_b: u8,
            field_b: u16,
            #[With(size(10))]
            name: String,
        }

        let data = TestComplexSize {
            field_a: a,
            bits_a: a & 0xF,
            bits_b: (a >> 4) & 0xF,
            field_b: b,
            name: s,
        };

        let bytes = data.to_be_bytes();

        // Total size: u8(1) + bits(1) + u16(2) + string(10) = 14 bytes
        prop_assert_eq!(bytes.len(), 14);
        prop_assert_eq!(TestComplexSize::field_size(), 14);
    }
}

// Array tests
proptest! {
    #[test]
    fn prop_array_round_trip(arr: [u8; 8]) {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestArray {
            data: [u8; 8],
        }

        let original = TestArray { data: arr };

        let bytes = original.to_be_bytes();
        let (decoded, _) = TestArray::try_from_be_bytes(&bytes).unwrap();

        prop_assert_eq!(&original, &decoded);
    }
}

// Option tests
proptest! {
    #[test]
    fn prop_option_round_trip(value: Option<u32>) {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestOption {
            maybe: Option<u32>,
        }

        let original = TestOption { maybe: value };

        let bytes = original.to_be_bytes();
        let (decoded, _) = TestOption::try_from_be_bytes(&bytes).unwrap();

        prop_assert_eq!(&original, &decoded);
    }
}

// Error handling tests
proptest! {
    #[test]
    fn prop_insufficient_data_error(bytes in prop::collection::vec(any::<u8>(), 0..3)) {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct TestU32 {
            field: u32,
        }

        // Trying to parse u32 from less than 4 bytes should fail
        let result = TestU32::try_from_be_bytes(&bytes);

        if bytes.is_empty() {
            prop_assert!(result.is_err());
            if let Err(e) = result {
                match e {
                    bebytes::BeBytesError::EmptyBuffer => {},
                    _ => prop_assert!(false, "Expected EmptyBuffer error"),
                }
            }
        } else {
            prop_assert!(result.is_err());
            if let Err(e) = result {
                match e {
                    bebytes::BeBytesError::InsufficientData { .. } => {},
                    _ => prop_assert!(false, "Expected InsufficientData error"),
                }
            }
        }
    }
}
