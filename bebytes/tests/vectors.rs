//! Vector handling tests for BeBytes
//!
//! This module tests:
//! - Fixed size vectors with #[With(size(N))]
//! - Dynamic size vectors with #[FromField(field_name)]
//! - Vectors as the last field
//! - Nested field access for vector sizes

use bebytes::BeBytes;

mod fixed_size_vectors {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq)]
    struct FixedVectorPacket {
        header: u32,
        #[With(size(16))]
        data: Vec<u8>,
        footer: u16,
    }

    #[test]
    fn test_fixed_size_vector() {
        let packet = FixedVectorPacket {
            header: 0xDEADBEEF,
            data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            footer: 0xCAFE,
        };

        let bytes = packet.to_be_bytes();
        assert_eq!(bytes.len(), 4 + 16 + 2);

        let (decoded, consumed) = FixedVectorPacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded, packet);
    }

    #[test]
    fn test_multiple_fixed_vectors() {
        #[derive(BeBytes, Debug, PartialEq)]
        struct MultipleVectors {
            #[With(size(4))]
            vec1: Vec<u8>,
            middle: u16,
            #[With(size(8))]
            vec2: Vec<u8>,
        }

        let packet = MultipleVectors {
            vec1: vec![0xAA, 0xBB, 0xCC, 0xDD],
            middle: 0x1234,
            vec2: vec![1, 2, 3, 4, 5, 6, 7, 8],
        };

        let bytes = packet.to_be_bytes();
        assert_eq!(bytes.len(), 4 + 2 + 8);

        let (decoded, _) = MultipleVectors::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded, packet);
    }
}

mod dynamic_size_vectors {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct DynamicVectorPacket {
        length: u16,
        #[FromField(length)]
        data: Vec<u8>,
        checksum: u32,
    }

    #[test]
    fn test_dynamic_size_from_field() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let packet = DynamicVectorPacket {
            length: data.len() as u16,
            data: data.clone(),
            checksum: 0x12345678,
        };

        let bytes = packet.to_be_bytes();
        assert_eq!(bytes.len(), 2 + 5 + 4);

        let (decoded, consumed) = DynamicVectorPacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded.length, 5);
        assert_eq!(decoded.data, data);
        assert_eq!(decoded.checksum, 0x12345678);
    }

    #[test]
    fn test_zero_length_dynamic_vector() {
        let packet = DynamicVectorPacket {
            length: 0,
            data: vec![],
            checksum: 0xDEADBEEF,
        };

        let bytes = packet.to_be_bytes();
        assert_eq!(bytes.len(), 2 + 4);

        let (decoded, _) = DynamicVectorPacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded.length, 0);
        assert_eq!(decoded.data.len(), 0);
    }
}

mod nested_field_access {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct Header {
        magic: u32,
        payload_length: u16,
        flags: u8,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct NestedFieldPacket {
        header: Header,
        sequence: u32,
        #[FromField(header.payload_length)]
        payload: Vec<u8>,
    }

    #[test]
    fn test_nested_field_vector_size() {
        let payload_data = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
        let packet = NestedFieldPacket {
            header: Header {
                magic: 0xDEADBEEF,
                payload_length: payload_data.len() as u16,
                flags: 0xFF,
            },
            sequence: 0x12345678,
            payload: payload_data.clone(),
        };

        let bytes = packet.to_be_bytes();
        let (decoded, _) = NestedFieldPacket::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded.header.payload_length, 5);
        assert_eq!(decoded.payload, payload_data);
    }

    #[test]
    fn test_deeply_nested_field_access() {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct InnerMost {
            count: u8,
        }

        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct Middle {
            inner: InnerMost,
            extra: u16,
        }

        #[derive(BeBytes, Debug, PartialEq)]
        struct Outer {
            middle: Middle,
            #[FromField(middle.inner.count)]
            data: Vec<u8>,
        }

        let data = vec![1, 2, 3, 4];
        let packet = Outer {
            middle: Middle {
                inner: InnerMost { count: 4 },
                extra: 0x1234,
            },
            data: data.clone(),
        };

        let bytes = packet.to_be_bytes();
        let (decoded, _) = Outer::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded.middle.inner.count, 4);
        assert_eq!(decoded.data, data);
    }
}

mod vector_as_last_field {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq)]
    struct PacketWithPadding {
        header: u32,
        packet_type: u8,
        // Vector without size consumes remaining bytes
        padding: Vec<u8>,
    }

    #[test]
    fn test_vector_as_padding() {
        let packet = PacketWithPadding {
            header: 0x12345678,
            packet_type: 0xAB,
            padding: vec![0xFF, 0xEE, 0xDD, 0xCC, 0xBB],
        };

        let bytes = packet.to_be_bytes();
        assert_eq!(bytes.len(), 4 + 1 + 5);

        let (decoded, consumed) = PacketWithPadding::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded, packet);
    }

    #[test]
    fn test_empty_padding_vector() {
        let bytes = vec![
            0x12, 0x34, 0x56, 0x78, // header
            0xAB, // packet_type
                  // No padding bytes
        ];

        let (decoded, consumed) = PacketWithPadding::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, 5);
        assert_eq!(decoded.header, 0x12345678);
        assert_eq!(decoded.packet_type, 0xAB);
        assert_eq!(decoded.padding.len(), 0);
    }
}

mod custom_type_vectors {
    use super::*;

    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct Item {
        id: u16,
        value: u32,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct ItemVector {
        count: u8,
        #[FromField(count)]
        items: Vec<Item>,
    }

    #[test]
    fn test_custom_type_vector() {
        let items = vec![
            Item {
                id: 1,
                value: 0xAABBCCDD,
            },
            Item {
                id: 2,
                value: 0x11223344,
            },
            Item {
                id: 3,
                value: 0xDEADBEEF,
            },
        ];

        let packet = ItemVector {
            count: items.len() as u8,
            items: items.clone(),
        };

        let bytes = packet.to_be_bytes();
        assert_eq!(bytes.len(), 1 + (3 * 6)); // count + 3 items of 6 bytes each

        let (decoded, _) = ItemVector::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded.count, 3);
        assert_eq!(decoded.items, items);
    }

    #[test]
    fn test_fixed_size_custom_vector() {
        #[derive(BeBytes, Debug, PartialEq)]
        struct FixedItemVector {
            header: u16,
            #[With(size(2))]
            items: Vec<Item>,
            footer: u16,
        }

        let packet = FixedItemVector {
            header: 0x1234,
            items: vec![
                Item {
                    id: 10,
                    value: 0xAAAAAAAA,
                },
                Item {
                    id: 20,
                    value: 0xBBBBBBBB,
                },
            ],
            footer: 0x5678,
        };

        let bytes = packet.to_be_bytes();
        let (decoded, _) = FixedItemVector::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(decoded.items.len(), 2);
        assert_eq!(decoded.items[0].id, 10);
        assert_eq!(decoded.items[1].id, 20);
    }
}
