//! Core functionality tests for BeBytes
//!
//! This module tests basic serialization/deserialization of:
//! - Primitive types
//! - Arrays
//! - Basic structs
//! - Nested structs

use bebytes::BeBytes;

mod primitive_types {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq)]
    struct AllPrimitives {
        a: u8,
        b: i8,
        c: u16,
        d: i16,
        e: u32,
        f: i32,
        g: u64,
        h: i64,
        i: u128,
        j: i128,
    }

    #[test]
    fn test_primitive_serialization() {
        let data = AllPrimitives {
            a: 255,
            b: -128,
            c: 65535,
            d: -32768,
            e: 4294967295,
            f: -2147483648,
            g: 18446744073709551615,
            h: -9223372036854775808,
            i: 340282366920938463463374607431768211455,
            j: -170141183460469231731687303715884105728,
        };

        // Test big-endian
        let be_bytes = data.to_be_bytes();
        let (decoded, consumed) = AllPrimitives::try_from_be_bytes(&be_bytes).unwrap();
        assert_eq!(consumed, be_bytes.len());
        assert_eq!(decoded, data);

        // Test little-endian
        let le_bytes = data.to_le_bytes();
        let (decoded, consumed) = AllPrimitives::try_from_le_bytes(&le_bytes).unwrap();
        assert_eq!(consumed, le_bytes.len());
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_size_calculation() {
        assert_eq!(
            AllPrimitives::field_size(),
            1 + 1 + 2 + 2 + 4 + 4 + 8 + 8 + 16 + 16
        );
    }
}

mod arrays {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq)]
    struct ArrayStruct {
        header: u32,
        data: [u8; 16],
        footer: u16,
    }

    #[test]
    fn test_array_serialization() {
        let data = ArrayStruct {
            header: 0xDEADBEEF,
            data: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            footer: 0xCAFE,
        };

        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 4 + 16 + 2);

        let (decoded, consumed) = ArrayStruct::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_zero_array() {
        let data = ArrayStruct {
            header: 0,
            data: [0; 16],
            footer: 0,
        };

        let bytes = data.to_be_bytes();
        let (decoded, _) = ArrayStruct::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, data);
    }
}

mod basic_structs {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq)]
    struct SimplePacket {
        version: u8,
        packet_type: u8,
        length: u16,
        sequence: u32,
    }

    #[test]
    fn test_simple_struct() {
        let packet = SimplePacket {
            version: 1,
            packet_type: 42,
            length: 1024,
            sequence: 0x12345678,
        };

        let bytes = packet.to_be_bytes();
        assert_eq!(bytes.len(), 8);

        // Verify byte layout
        assert_eq!(bytes[0], 1); // version
        assert_eq!(bytes[1], 42); // packet_type
        assert_eq!(bytes[2], 0x04); // length high byte
        assert_eq!(bytes[3], 0x00); // length low byte
        assert_eq!(bytes[4], 0x12); // sequence bytes
        assert_eq!(bytes[5], 0x34);
        assert_eq!(bytes[6], 0x56);
        assert_eq!(bytes[7], 0x78);

        let (decoded, _) = SimplePacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn test_endianness_consistency() {
        let packet = SimplePacket {
            version: 1,
            packet_type: 2,
            length: 0x0304,
            sequence: 0x05060708,
        };

        // Big-endian
        let be_bytes = packet.to_be_bytes();
        assert_eq!(be_bytes[2..4], [0x03, 0x04]); // length in network order
        assert_eq!(be_bytes[4..8], [0x05, 0x06, 0x07, 0x08]); // sequence in network order

        // Little-endian
        let le_bytes = packet.to_le_bytes();
        assert_eq!(le_bytes[2..4], [0x04, 0x03]); // length in little-endian
        assert_eq!(le_bytes[4..8], [0x08, 0x07, 0x06, 0x05]); // sequence in little-endian
    }
}

mod nested_structs {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct Header {
        magic: u32,
        version: u8,
        flags: u8,
        length: u16,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct Packet {
        header: Header,
        payload_type: u32,
        checksum: u16,
    }

    #[test]
    fn test_nested_struct_serialization() {
        let packet = Packet {
            header: Header {
                magic: 0xDEADBEEF,
                version: 1,
                flags: 0xFF,
                length: 1024,
            },
            payload_type: 0xCAFEBABE,
            checksum: 0x1234,
        };

        let bytes = packet.to_be_bytes();
        assert_eq!(bytes.len(), 8 + 4 + 2); // header + payload_type + checksum

        let (decoded, consumed) = Packet::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded, packet);
        assert_eq!(decoded.header.magic, 0xDEADBEEF);
        assert_eq!(decoded.header.version, 1);
        assert_eq!(decoded.header.flags, 0xFF);
        assert_eq!(decoded.header.length, 1024);
    }

    #[test]
    fn test_deeply_nested() {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct Inner {
            value: u16,
        }

        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct Middle {
            inner: Inner,
            extra: u8,
        }

        #[derive(BeBytes, Debug, PartialEq)]
        struct Outer {
            middle: Middle,
            final_value: u32,
        }

        let data = Outer {
            middle: Middle {
                inner: Inner { value: 0x1234 },
                extra: 0xAB,
            },
            final_value: 0xDEADBEEF,
        };

        let bytes = data.to_be_bytes();
        let (decoded, _) = Outer::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, data);
        assert_eq!(decoded.middle.inner.value, 0x1234);
    }
}

mod edge_cases {
    use super::*;

    // Empty structs not currently supported
    // #[derive(BeBytes, Debug, PartialEq)]
    // struct EmptyStruct {}

    #[derive(BeBytes, Debug, PartialEq)]
    struct SingleByte {
        value: u8,
    }

    // Empty structs currently not supported - they fail with EmptyBuffer error
    // #[test]
    // fn test_empty_struct() {
    //     let data = EmptyStruct {};
    //     let bytes = data.to_be_bytes();
    //     assert_eq!(bytes.len(), 0);
    //
    //     let (decoded, consumed) = EmptyStruct::try_from_be_bytes(&bytes).unwrap();
    //     assert_eq!(consumed, 0);
    //     assert_eq!(decoded, data);
    // }

    #[test]
    fn test_single_byte_struct() {
        let data = SingleByte { value: 42 };
        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 42);

        let (decoded, consumed) = SingleByte::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, 1);
        assert_eq!(decoded, data);
    }
}
