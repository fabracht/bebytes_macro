use bebytes::{BeBytes, FixedString};

/// Demo test showing practical usage of FixedString in network protocol scenarios
#[test]
fn test_network_packet_with_fixed_strings() {
    // Example: Network packet with fixed-size string fields
    #[derive(BeBytes, Debug, PartialEq)]
    struct NetworkPacket {
        version: u8,
        #[bits(4)]
        flags: u8,
        #[bits(4)]
        reserved: u8,
        sender: FixedString<32>,       // 32-byte sender name
        receiver: FixedString<32>,     // 32-byte receiver name
        message_type: FixedString<16>, // 16-byte message type
        sequence: u32,
        payload_size: u16,
        #[FromField(payload_size)]
        payload: Vec<u8>,
    }

    let packet = NetworkPacket {
        version: 1,
        flags: 15,
        reserved: 0,
        sender: FixedString::from_str("alice@example.com"),
        receiver: FixedString::from_str("bob@example.com"),
        message_type: FixedString::from_str("chat"),
        sequence: 12345,
        payload_size: 5,
        payload: vec![b'h', b'e', b'l', b'l', b'o'],
    };

    // Serialize
    let bytes = packet.to_be_bytes();

    // Expected size: 1 + 1 + 32 + 32 + 16 + 4 + 2 + 5 = 93 bytes
    assert_eq!(bytes.len(), 93);

    // Deserialize
    let (decoded, bytes_read) = NetworkPacket::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(bytes_read, 93);
    assert_eq!(decoded, packet);

    // Verify string fields
    assert_eq!(decoded.sender.as_str(), Some("alice@example.com"));
    assert_eq!(decoded.receiver.as_str(), Some("bob@example.com"));
    assert_eq!(decoded.message_type.as_str(), Some("chat"));
    assert_eq!(decoded.payload, vec![b'h', b'e', b'l', b'l', b'o']);
}

#[test]
fn test_database_record_with_fixed_strings() {
    // Example: Database record with fixed-size fields
    #[derive(BeBytes, Debug, PartialEq)]
    struct UserRecord {
        id: u64,
        username: FixedString<20>,
        email: FixedString<50>,
        first_name: FixedString<25>,
        last_name: FixedString<25>,
        is_active: u8, // 0 or 1
        created_timestamp: u64,
    }

    let record = UserRecord {
        id: 123456789,
        username: FixedString::from_str("johndoe"),
        email: FixedString::from_str("john.doe@example.com"),
        first_name: FixedString::from_str("John"),
        last_name: FixedString::from_str("Doe"),
        is_active: 1,
        created_timestamp: 1640995200, // 2022-01-01 00:00:00 UTC
    };

    let bytes = record.to_be_bytes();

    // Expected size: 8 + 20 + 50 + 25 + 25 + 1 + 8 = 137 bytes
    assert_eq!(bytes.len(), 137);

    let (decoded, _) = UserRecord::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded, record);

    // Verify all string fields
    assert_eq!(decoded.username.as_str(), Some("johndoe"));
    assert_eq!(decoded.email.as_str(), Some("john.doe@example.com"));
    assert_eq!(decoded.first_name.as_str(), Some("John"));
    assert_eq!(decoded.last_name.as_str(), Some("Doe"));
}

#[test]
fn test_mixed_field_types_with_strings() {
    // Example: Complex struct mixing various field types including FixedString
    #[derive(BeBytes, Debug, PartialEq)]
    struct ComplexMessage {
        #[bits(4)]
        version: u8,
        #[bits(4)]
        priority: u8,
        sender_id: u32,
        message_id: FixedString<16>,
        timestamp: u64,
        category: FixedString<8>,
        tags: [u8; 4], // 4 bytes for tag flags
        content_length: u16,
        #[FromField(content_length)]
        content: Vec<u8>,
    }

    let message = ComplexMessage {
        version: 2,
        priority: 7,
        sender_id: 999,
        message_id: FixedString::from_str("MSG-2024-001"),
        timestamp: 1704067200,
        category: FixedString::from_str("urgent"),
        tags: [0x01, 0x04, 0x08, 0x10],
        content_length: 13,
        content: b"Hello, world!".to_vec(),
    };

    let bytes = message.to_be_bytes();
    let (decoded, _) = ComplexMessage::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(decoded, message);
    assert_eq!(decoded.message_id.as_str(), Some("MSG-2024-001"));
    assert_eq!(decoded.category.as_str(), Some("urgent"));
    assert_eq!(decoded.content, b"Hello, world!");
}

#[test]
fn test_protocol_header_with_fixed_strings() {
    // Example: Protocol header similar to HTTP or custom protocols
    #[derive(BeBytes, Debug, PartialEq)]
    struct ProtocolHeader {
        magic: [u8; 4], // Protocol magic number
        version: u16,
        method: FixedString<8>,        // GET, POST, PUT, etc.
        path: FixedString<64>,         // URL path
        content_type: FixedString<32>, // MIME type
        content_length: u32,
    }

    let header = ProtocolHeader {
        magic: *b"HTTP",
        version: 0x0101, // HTTP/1.1
        method: FixedString::from_str("GET"),
        path: FixedString::from_str("/api/v1/users"),
        content_type: FixedString::from_str("application/json"),
        content_length: 256,
    };

    let bytes = header.to_be_bytes();

    // Expected size: 4 + 2 + 8 + 64 + 32 + 4 = 114 bytes
    assert_eq!(bytes.len(), 114);

    let (decoded, _) = ProtocolHeader::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded, header);

    // Verify string fields
    assert_eq!(decoded.method.as_str(), Some("GET"));
    assert_eq!(decoded.path.as_str(), Some("/api/v1/users"));
    assert_eq!(decoded.content_type.as_str(), Some("application/json"));
}

#[test]
fn test_fixed_string_with_unicode_in_real_scenario() {
    // Example: International user data
    #[derive(BeBytes, Debug, PartialEq)]
    struct InternationalUser {
        id: u32,
        name: FixedString<40>, // Support for longer Unicode names
        city: FixedString<30>,
        country_code: FixedString<3>, // ISO country code
        language: FixedString<6>,     // Language code
    }

    let user = InternationalUser {
        id: 12345,
        name: FixedString::from_str("José María García"),
        city: FixedString::from_str("北京"), // Beijing in Chinese
        country_code: FixedString::from_str("CN"),
        language: FixedString::from_str("zh-CN"),
    };

    let bytes = user.to_be_bytes();
    let (decoded, _) = InternationalUser::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(decoded, user);
    assert_eq!(decoded.name.as_str(), Some("José María García"));
    assert_eq!(decoded.city.as_str(), Some("北京"));
    assert_eq!(decoded.country_code.as_str(), Some("CN"));
    assert_eq!(decoded.language.as_str(), Some("zh-CN"));
}

#[test]
fn test_fixed_string_edge_cases_in_protocols() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct EdgeCaseTest {
        empty_string: FixedString<10>,
        max_length: FixedString<5>,
        with_nulls: FixedString<8>,
    }

    let test = EdgeCaseTest {
        empty_string: FixedString::from_str(""),
        max_length: FixedString::from_str("12345"), // Exactly 5 bytes
        with_nulls: {
            let mut fs = FixedString::<8>::from_str("test");
            // Manually set some data to test null handling
            fs
        },
    };

    let bytes = test.to_be_bytes();
    let (decoded, _) = EdgeCaseTest::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(decoded, test);
    assert_eq!(decoded.empty_string.as_str(), Some(""));
    assert_eq!(decoded.max_length.as_str(), Some("12345"));
    assert_eq!(decoded.with_nulls.as_str(), Some("test"));

    // Verify lengths
    assert_eq!(decoded.empty_string.len(), 0);
    assert_eq!(decoded.max_length.len(), 5);
    assert_eq!(decoded.with_nulls.len(), 4);
}
