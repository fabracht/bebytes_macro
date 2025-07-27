//! Integration tests for BeBytes
//!
//! This module tests complex real-world scenarios:
//! - Complete packet workflows
//! - TLV (Type-Length-Value) structures
//! - Complex nested structures with multiple features
//! - Real protocol implementations

use bebytes::BeBytes;

mod packet_protocols {
    use super::*;

    // Simulated network packet protocol
    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
    enum PacketType {
        Data = 0x01,
        Ack = 0x02,
        Control = 0x03,
        Heartbeat = 0x04,
    }

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
    #[bebytes(flags)]
    enum PacketFlags {
        None = 0,
        Urgent = 1,
        Fragmented = 2,
        Encrypted = 4,
        Compressed = 8,
    }

    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct PacketHeader {
        magic: u32,  // 0xDEADBEEF
        version: u8, // Protocol version
        packet_type: PacketType,
        flags: u8, // Bitwise combination of PacketFlags
        sequence_number: u32,
        payload_length: u16,
        checksum: u16,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct CompletePacket {
        header: PacketHeader,
        #[FromField(header.payload_length)]
        payload: Vec<u8>,
    }

    #[test]
    fn test_complete_packet_workflow() {
        // Create a data packet
        let payload_data = b"Hello, BeBytes!".to_vec();
        let packet = CompletePacket {
            header: PacketHeader {
                magic: 0xDEADBEEF,
                version: 1,
                packet_type: PacketType::Data,
                flags: PacketFlags::Urgent | PacketFlags::Compressed,
                sequence_number: 12345,
                payload_length: payload_data.len() as u16,
                checksum: 0xABCD, // Simplified checksum
            },
            payload: payload_data.clone(),
        };

        // Serialize
        let bytes = packet.to_be_bytes();
        assert_eq!(bytes.len(), 15 + payload_data.len()); // Header (15 bytes) + payload

        // Deserialize
        let (decoded, consumed) = CompletePacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded, packet);

        // Verify individual fields
        assert_eq!(decoded.header.magic, 0xDEADBEEF);
        assert_ne!(decoded.header.flags & (PacketFlags::Urgent as u8), 0);
        assert_ne!(decoded.header.flags & (PacketFlags::Compressed as u8), 0);
        assert_eq!(decoded.payload, payload_data);
    }

    #[test]
    fn test_packet_fragmentation() {
        // Simulate fragmented packet handling
        #[derive(BeBytes, Debug, PartialEq)]
        struct FragmentedPacket {
            header: PacketHeader,
            fragment_offset: u32,
            fragment_total: u16,
            fragment_index: u16,
            #[FromField(header.payload_length)]
            fragment_data: Vec<u8>,
        }

        let fragment = FragmentedPacket {
            header: PacketHeader {
                magic: 0xDEADBEEF,
                version: 1,
                packet_type: PacketType::Data,
                flags: PacketFlags::Fragmented as u8,
                sequence_number: 100,
                payload_length: 512,
                checksum: 0x1234,
            },
            fragment_offset: 1024,
            fragment_total: 4,
            fragment_index: 2,
            fragment_data: vec![0xAA; 512],
        };

        let bytes = fragment.to_be_bytes();
        let (decoded, _) = FragmentedPacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded.fragment_index, 2);
        assert_eq!(decoded.fragment_data.len(), 512);
    }
}

mod tlv_structures {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
    enum TlvType {
        Integer = 0x01,
        String = 0x02,
        Binary = 0x03,
        Nested = 0x04,
    }

    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct TlvField {
        field_type: TlvType,
        length: u16,
        #[FromField(length)]
        value: Vec<u8>,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct TlvMessage {
        message_id: u32,
        field_count: u16,
        #[FromField(field_count)]
        fields: Vec<TlvField>,
    }

    #[test]
    fn test_tlv_structure() {
        let message = TlvMessage {
            message_id: 0x12345678,
            field_count: 3,
            fields: vec![
                TlvField {
                    field_type: TlvType::Integer,
                    length: 4,
                    value: vec![0x00, 0x00, 0x00, 0x42], // 66 in big-endian
                },
                TlvField {
                    field_type: TlvType::String,
                    length: 5,
                    value: b"hello".to_vec(),
                },
                TlvField {
                    field_type: TlvType::Binary,
                    length: 8,
                    value: vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE],
                },
            ],
        };

        let bytes = message.to_be_bytes();
        let (decoded, consumed) = TlvMessage::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded.field_count, 3);
        assert_eq!(decoded.fields.len(), 3);
        assert_eq!(decoded.fields[1].field_type, TlvType::String);
        assert_eq!(decoded.fields[1].value, b"hello");
    }

    #[test]
    fn test_nested_tlv() {
        // Test TLV containing other TLV structures
        let inner_tlv = TlvField {
            field_type: TlvType::Integer,
            length: 4,
            value: vec![0x00, 0x00, 0x01, 0x00], // 256
        };

        let inner_bytes = inner_tlv.to_be_bytes();

        let outer_tlv = TlvField {
            field_type: TlvType::Nested,
            length: inner_bytes.len() as u16,
            value: inner_bytes,
        };

        let bytes = outer_tlv.to_be_bytes();
        let (decoded, _) = TlvField::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded.field_type, TlvType::Nested);

        // Parse the nested TLV from the value
        let (nested, _) = TlvField::try_from_be_bytes(&decoded.value).unwrap();
        assert_eq!(nested.field_type, TlvType::Integer);
        assert_eq!(nested.value, vec![0x00, 0x00, 0x01, 0x00]);
    }
}

mod complex_protocols {
    use super::*;

    // Simulate a complex protocol with mixed features
    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
    enum CommandType {
        Read = 0x10,
        Write = 0x20,
        Delete = 0x30,
        Query = 0x40,
    }

    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct CommandHeader {
        #[bits(4)]
        version: u8,
        #[bits(4)]
        priority: u8,
        command: CommandType,
        #[bits(1)]
        has_auth: u8,
        #[bits(1)]
        has_payload: u8,
        #[bits(6)]
        reserved: u8,
        transaction_id: u32,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct AuthInfo {
        method: u8,
        token: [u8; 32],
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct Command {
        header: CommandHeader,
        payload_length: u32,
        #[FromField(payload_length)]
        payload: Vec<u8>,
        crc32: u32,
    }

    #[test]
    fn test_complex_command_structure() {
        let command_data = b"SELECT * FROM users WHERE id = 42".to_vec();

        let command = Command {
            header: CommandHeader {
                version: 2,
                priority: 15,
                command: CommandType::Query,
                has_auth: 1,
                has_payload: 1,
                reserved: 0,
                transaction_id: 0x9876FEDC,
            },
            payload_length: command_data.len() as u32,
            payload: command_data.clone(),
            crc32: 0x12345678, // Simplified CRC
        };

        let bytes = command.to_be_bytes();

        // Verify the command structure
        let (decoded, consumed) = Command::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded.header.version, 2);
        assert_eq!(decoded.header.priority, 15);
        assert_eq!(decoded.header.has_auth, 1);
        assert_eq!(decoded.payload, command_data);
    }
}

mod performance_scenarios {
    use super::*;

    // Large data structure to test performance
    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct SensorReading {
        timestamp: u64,
        sensor_id: u16,
        temperature: i16,
        humidity: u8,
        pressure: u32,
        #[bits(4)]
        quality: u8,
        #[bits(4)]
        status: u8,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct SensorBatch {
        batch_id: u64,
        sensor_count: u32,
        #[FromField(sensor_count)]
        readings: Vec<SensorReading>,
        checksum: u64,
    }

    #[test]
    fn test_large_batch_processing() {
        let mut readings = Vec::new();
        for i in 0..100 {
            readings.push(SensorReading {
                timestamp: 1234567890 + i as u64,
                sensor_id: (i % 10) as u16,
                temperature: (200 + (i as i16 * 5)) % 400 - 200,
                humidity: (40 + i) as u8 % 100,
                pressure: 101325 + (i * 100),
                quality: (i % 16) as u8,
                status: ((i / 16) % 16) as u8,
            });
        }

        let batch = SensorBatch {
            batch_id: 0xDEADBEEFCAFEBABE,
            sensor_count: readings.len() as u32,
            readings: readings.clone(),
            checksum: 0x123456789ABCDEF0,
        };

        let bytes = batch.to_be_bytes();
        assert!(bytes.len() > 1000); // Should be substantial size

        let (decoded, consumed) = SensorBatch::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded.sensor_count, 100);
        assert_eq!(decoded.readings.len(), 100);

        // Spot check some readings
        assert_eq!(decoded.readings[0], readings[0]);
        assert_eq!(decoded.readings[50], readings[50]);
        assert_eq!(decoded.readings[99], readings[99]);
    }
}

mod edge_case_integration {
    use super::*;

    // Test structure with all supported features
    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
    #[bebytes(flags)]
    enum FeatureFlags {
        None = 0,
        FeatureA = 1,
        FeatureB = 2,
        FeatureC = 4,
        FeatureD = 8,
    }

    #[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
    #[repr(u8)]
    enum Mode {
        Normal = 0,
        Extended = 1,
        Compact = 2,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct KitchenSink {
        // Primitive types
        u8_field: u8,
        i16_field: i16,
        u32_field: u32,

        // Arrays
        fixed_array: [u8; 4],

        // Bit fields
        #[bits(3)]
        small_bits: u8,
        #[bits(13)]
        medium_bits: u16,

        // Mode as u8 (2 bits for 3 values: Normal=0, Extended=1, Compact=2)
        #[bits(2)]
        mode: u8,
        // Padding to complete the byte (3 + 13 + 2 + 6 = 24 bits = 3 bytes)
        #[bits(6)]
        padding_bits: u8,

        // Flag enum (stored as u8 due to bitwise operations)
        flags: u8,

        // Options
        optional_value: Option<u32>,

        // Nested struct
        nested: ComplexNested,

        // Vectors
        vec_length: u16,
        #[FromField(vec_length)]
        dynamic_vec: Vec<u8>,

        #[With(size(8))]
        fixed_vec: Vec<u8>,

        // Vector as last field
        padding: Vec<u8>,
    }

    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct ComplexNested {
        inner_value: u64,
        inner_array: [u8; 3],
    }

    #[test]
    fn test_kitchen_sink() {
        let data = KitchenSink {
            u8_field: 0xFF,
            i16_field: -12345,
            u32_field: 0xDEADBEEF,
            fixed_array: [0x01, 0x02, 0x03, 0x04],
            small_bits: 0x07,
            medium_bits: 0x1FFF,
            mode: 1,         // 1 = Mode::Extended
            padding_bits: 0, // Padding
            flags: FeatureFlags::FeatureA | FeatureFlags::FeatureC,
            optional_value: Some(0xCAFEBABE),
            nested: ComplexNested {
                inner_value: 0x123456789ABCDEF0,
                inner_array: [0xAA, 0xBB, 0xCC],
            },
            vec_length: 5,
            dynamic_vec: vec![0x10, 0x20, 0x30, 0x40, 0x50],
            fixed_vec: vec![0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8],
            padding: vec![0xFF, 0xEE, 0xDD],
        };

        let bytes = data.to_be_bytes();
        let (decoded, consumed) = KitchenSink::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded, data);

        // Verify specific fields
        assert_eq!(decoded.small_bits, 0x07);
        assert_eq!(decoded.mode, 1); // 1 = Mode::Extended
                                     // Check flags using bitwise operations since flags is u8
        assert_eq!(
            decoded.flags & (FeatureFlags::FeatureA as u8),
            FeatureFlags::FeatureA as u8
        );
        assert_eq!(
            decoded.flags & (FeatureFlags::FeatureC as u8),
            FeatureFlags::FeatureC as u8
        );
        assert!(decoded.optional_value.is_some());
        assert_eq!(decoded.dynamic_vec.len(), 5);
        assert_eq!(decoded.padding, vec![0xFF, 0xEE, 0xDD]);
    }
}
