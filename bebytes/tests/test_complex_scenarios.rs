use bebytes::BeBytes;

// Integration test for complex real-world scenarios

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
enum PacketType {
    Data = 0,
    Control = 1,
    Ack = 2,
    ErrorPacket = 3,
}

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[bebytes(flags)]
#[repr(u8)]
enum PacketFlags {
    None = 0,
    Urgent = 1,
    Encrypted = 2,
    Compressed = 4,
    Fragmented = 8,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct PacketHeader {
    #[bits(4)]
    version: u8,
    #[bits()] // Auto-sized to 2 bits
    packet_type: PacketType,
    #[bits(2)]
    reserved: u8,
    flags: u8, // Stores PacketFlags
    sequence_number: u16,
    payload_length: u16,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct Packet {
    header: PacketHeader,
    payload_length: u16,
    #[FromField(payload_length)]
    payload: Vec<u8>,
    checksum: u32,
}

#[test]
fn test_complete_packet_workflow() {
    // Create a complex packet
    let packet = Packet {
        header: PacketHeader {
            version: 1,
            packet_type: PacketType::Data,
            reserved: 0,
            flags: PacketFlags::Encrypted as u8 | PacketFlags::Compressed as u8,
            sequence_number: 12345,
            payload_length: 10,
        },
        payload_length: 10,
        payload: vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0xFF],
        checksum: 0x12345678,
    };

    // Serialize
    let bytes = packet.to_be_bytes();

    // Expected layout:
    // - 1 byte: version(4) + packet_type(2) + reserved(2)
    // - 1 byte: flags
    // - 2 bytes: sequence_number (BE)
    // - 2 bytes: payload_length (BE)
    // - 2 bytes: payload_length (BE) - duplicate field
    // - 10 bytes: payload
    // - 4 bytes: checksum (BE)
    // Total: 22 bytes
    assert_eq!(bytes.len(), 22);

    // Verify header bits
    assert_eq!(bytes[0] & 0xF0, 0x10); // version = 1 << 4
    assert_eq!(bytes[0] & 0x0C, 0x00); // packet_type = 0 << 2
    assert_eq!(bytes[0] & 0x03, 0x00); // reserved = 0

    // Verify flags
    assert_eq!(bytes[1], 6); // Encrypted(2) | Compressed(4) = 6

    // Deserialize
    let (decoded, consumed) = Packet::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, 22);
    assert_eq!(decoded, packet);

    // Verify individual fields
    assert_eq!(decoded.header.version, 1);
    assert_eq!(decoded.header.packet_type, PacketType::Data);
    assert_eq!(
        decoded.header.flags & PacketFlags::Encrypted as u8,
        PacketFlags::Encrypted as u8
    );
    assert_eq!(
        decoded.header.flags & PacketFlags::Compressed as u8,
        PacketFlags::Compressed as u8
    );
    assert_eq!(decoded.header.sequence_number, 12345);
    assert_eq!(decoded.payload_length, 10);
    assert_eq!(decoded.payload.len(), 10);
    assert_eq!(decoded.checksum, 0x12345678);
}

#[test]
fn test_tlv_field_structure() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct TlvField {
        tag: u8,
        length: u8,
        #[FromField(length)]
        value: Vec<u8>,
    }

    let field = TlvField {
        tag: 1,
        length: 4,
        value: vec![0x01, 0x02, 0x03, 0x04],
    };

    let bytes = field.to_be_bytes();
    assert_eq!(bytes.len(), 6); // 1 + 1 + 4
    assert_eq!(bytes[0], 1); // tag
    assert_eq!(bytes[1], 4); // length
    assert_eq!(&bytes[2..], &[0x01, 0x02, 0x03, 0x04]); // value

    let (decoded, consumed) = TlvField::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, 6);
    assert_eq!(decoded, field);
    assert_eq!(decoded.value.len(), 4);
}
