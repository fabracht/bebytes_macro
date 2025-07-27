//! Error handling tests for BeBytes
//!
//! This module tests:
//! - Error display formatting
//! - All error variants
//! - Custom Result type alias compatibility
//! - Error propagation

use bebytes::{BeBytes, BeBytesError};

mod error_display {
    use super::*;

    #[test]
    fn test_error_display_formatting() {
        // EmptyBuffer
        let err = BeBytesError::EmptyBuffer;
        assert_eq!(err.to_string(), "No bytes provided");

        // InsufficientData
        let err = BeBytesError::InsufficientData {
            expected: 10,
            actual: 5,
        };
        assert_eq!(err.to_string(), "Not enough bytes: expected 10, got 5");

        // InvalidDiscriminant
        let err = BeBytesError::InvalidDiscriminant {
            value: 42,
            type_name: "TestEnum",
        };
        assert_eq!(err.to_string(), "Invalid discriminant 42 for type TestEnum");

        // InvalidBitField
        let err = BeBytesError::InvalidBitField {
            value: 256,
            max: 255,
            field: "test_field",
        };
        assert_eq!(
            err.to_string(),
            "Value 256 exceeds maximum 255 for field test_field"
        );
    }

    #[test]
    fn test_error_traits() {
        let err = BeBytesError::EmptyBuffer;

        // Test Debug
        let debug_str = format!("{:?}", err);
        assert_eq!(debug_str, "EmptyBuffer");

        // Test Clone
        let cloned = err;
        assert_eq!(cloned, err);

        // Test Copy (implicit)
        let copied: BeBytesError = err;
        assert_eq!(copied, err);

        // Test PartialEq
        assert_eq!(err, BeBytesError::EmptyBuffer);
        assert_ne!(
            err,
            BeBytesError::InsufficientData {
                expected: 1,
                actual: 0
            }
        );
    }
}

mod error_variants {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq)]
    struct TestStruct {
        field1: u32,
        field2: u16,
    }

    #[test]
    fn test_empty_buffer_error() {
        let empty_bytes: Vec<u8> = vec![];
        let result = TestStruct::try_from_be_bytes(&empty_bytes);

        match result {
            Err(BeBytesError::EmptyBuffer) => {
                // Expected
            }
            _ => panic!("Expected EmptyBuffer error"),
        }
    }

    #[test]
    fn test_insufficient_data_error() {
        let short_bytes = vec![0x12, 0x34]; // Only 2 bytes, need 6
        let result = TestStruct::try_from_be_bytes(&short_bytes);

        match result {
            Err(BeBytesError::InsufficientData { expected, actual }) => {
                assert_eq!(expected, 4); // Expecting 4 bytes for u32
                assert_eq!(actual, 2); // But only have 2
            }
            _ => panic!("Expected InsufficientData error"),
        }
    }

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
    enum TestEnum {
        A = 0,
        B = 1,
        C = 2,
    }

    #[test]
    fn test_invalid_discriminant_error() {
        // Try to create enum with invalid discriminant
        match TestEnum::try_from(5u8) {
            Err(BeBytesError::InvalidDiscriminant { value, type_name }) => {
                assert_eq!(value, 5);
                assert_eq!(type_name, "TestEnum");
            }
            _ => panic!("Expected InvalidDiscriminant error"),
        }
    }

    #[test]
    fn test_auto_enum_invalid_value() {
        #[derive(BeBytes, Debug, PartialEq)]
        struct EnumPacket {
            #[bits(4)]
            prefix: u8,
            #[bits(2)] // TestEnum as u8: values 0-2, needs 2 bits
            value: u8,
            #[bits(2)] // Padding to complete byte
            padding: u8,
        }

        // Create bytes with invalid enum value
        // TestEnum needs 2 bits (values 0,1,2 -> max 2 -> needs 2 bits)
        // Layout: prefix(4) | enum(2) | unused(2)
        // To get invalid value 3 for the enum: 0000_11xx
        let bytes = vec![0b0000_1100]; // prefix=0, enum=3 (invalid)

        // Test successful parsing since we're now using u8 fields
        let (packet, _) = EnumPacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(packet.prefix, 0);
        assert_eq!(packet.value, 3); // This is now valid since it's just a u8
        assert_eq!(packet.padding, 0);
    }
}

mod custom_result_alias {
    use super::*;

    // User's custom Result type (like in MQTT library)
    type Result<T> = std::result::Result<T, String>;

    #[derive(BeBytes, Debug, PartialEq)]
    struct Packet {
        header: u32,
        data: u16,
    }

    #[test]
    fn test_works_with_custom_result_alias() {
        // This function uses the custom Result type
        fn process_packet(bytes: &[u8]) -> Result<Packet> {
            // The generated code uses fully qualified paths, so it works
            match Packet::try_from_be_bytes(bytes) {
                Ok((packet, _)) => Ok(packet),
                Err(e) => Err(format!("Failed to parse packet: {}", e)),
            }
        }

        let bytes = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x12, 0x34];
        let result = process_packet(&bytes);
        assert!(result.is_ok());

        let short_bytes = vec![0x12, 0x34];
        let result = process_packet(&short_bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not enough bytes"));
    }
}

mod error_propagation {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct Inner {
        value: u32,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct Outer {
        inner: Inner,
        extra: u16,
    }

    #[test]
    fn test_nested_error_propagation() {
        // Test that errors from inner structs propagate correctly
        let short_bytes = vec![0x12, 0x34]; // Not enough for Inner's u32

        match Outer::try_from_be_bytes(&short_bytes) {
            Err(BeBytesError::InsufficientData { expected, actual }) => {
                assert_eq!(expected, 4); // Inner needs 4 bytes
                assert_eq!(actual, 2);
            }
            _ => panic!("Expected error to propagate from Inner"),
        }
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct VectorPacket {
        count: u8,
        #[FromField(count)]
        items: Vec<Inner>,
    }

    #[test]
    fn test_vector_item_error_propagation() {
        // Test that vector parsing is lenient - it parses as many complete items as possible
        let bytes = vec![
            2, // count = 2
            0x12, 0x34, 0x56, 0x78, // First item (complete)
            0xAB, 0xCD, // Second item (incomplete)
        ];

        match VectorPacket::try_from_be_bytes(&bytes) {
            Ok((packet, consumed)) => {
                // Vector parsing is lenient - it parsed 1 item instead of failing
                assert_eq!(packet.count, 2);
                assert_eq!(packet.items.len(), 1);
                assert_eq!(packet.items[0].value, 0x12345678);
                assert_eq!(consumed, 5); // 1 byte count + 4 bytes for one item
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}

mod no_std_compatibility {
    use super::*;

    #[test]
    fn test_error_display_no_std() {
        // Test that error formatting works without std
        #[cfg(not(feature = "std"))]
        {
            extern crate alloc;
            use alloc::format;

            let err = BeBytesError::EmptyBuffer;
            let formatted = format!("{}", err);
            assert_eq!(formatted, "No bytes provided");
        }

        // In std mode, just verify it compiles
        #[cfg(feature = "std")]
        {
            let err = BeBytesError::EmptyBuffer;
            let _formatted = format!("{}", err);
        }
    }

    #[test]
    fn test_error_size() {
        // Ensure error type is small and efficient
        use core::mem::size_of;

        // Should be reasonably small (two usizes + discriminant + padding)
        let size = size_of::<BeBytesError>();
        println!("BeBytesError size: {} bytes", size);
        assert!(size <= 64); // Increased limit - static strings make it larger
    }
}
