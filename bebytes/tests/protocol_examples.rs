use bebytes::BeBytes as _;
use bebytes_derive::BeBytes;

/// IPv4 packet header
#[derive(BeBytes, Debug, PartialEq)]
struct Ipv4Packet {
    version: u8,
    header_length: u8,
    type_of_service: u8,
    total_length: u16,
    identification: u16,
    flags_and_fragment: u16,
    ttl: u8,
    protocol: u8,
    checksum: u16,
    // IPv4 addresses are always 4 bytes
    #[With(size(4))]
    source_address: Vec<u8>,
    #[With(size(4))]
    dest_address: Vec<u8>,
}

/// IPv6 packet header (simplified)  
#[derive(BeBytes, Debug, PartialEq)]
struct Ipv6Packet {
    version: u8,
    traffic_class: u8,
    flow_label_high: u8,
    flow_label_low: u16,
    payload_length: u16,
    next_header: u8,
    hop_limit: u8,
    // IPv6 addresses are always 16 bytes
    #[With(size(16))]
    source_address: Vec<u8>,
    #[With(size(16))]
    dest_address: Vec<u8>,
}

/// DNS message with variable-length question and answer sections
#[derive(BeBytes, Debug, PartialEq)]
struct DnsMessage {
    id: u16,
    flags: u16,
    question_count: u16,
    answer_count: u16,
    authority_count: u16,
    additional_count: u16,
    // Variable-length data sections
    #[With(size(question_count * 5))] // Simplified: each question ~5 bytes
    questions: Vec<u8>,
    #[With(size(answer_count * 12))] // Simplified: each answer ~12 bytes
    answers: Vec<u8>,
}

/// MQTT Control Packet with variable remaining length
#[derive(BeBytes, Debug, PartialEq)]
struct MqttPacket {
    fixed_header: u8,
    remaining_length: u8,
    // Payload size determined by remaining length
    #[With(size(remaining_length))]
    payload: Vec<u8>,
}

/// TCP segment with variable options length
#[derive(BeBytes, Debug, PartialEq)]
struct TcpSegment {
    source_port: u16,
    dest_port: u16,
    sequence_number: u32,
    ack_number: u32,
    data_offset_and_flags: u16,
    window_size: u16,
    checksum: u16,
    urgent_pointer: u16,
    // Options length = (data_offset >> 12) * 4 - 20 (base header size)
    // Simplified for testing: options_length field
    options_length: u8,
    #[With(size(options_length))]
    options: Vec<u8>,
    // Remaining data
    data_length: u16,
    #[With(size(data_length))]
    data: Vec<u8>,
}

/// HTTP-like message with content-length header
#[derive(BeBytes, Debug, PartialEq)]
struct HttpMessage {
    status_code: u16,
    header_count: u8,
    #[With(size(header_count * 32))] // Each header ~32 bytes
    headers: String,
    content_length: u32,
    #[With(size(content_length))]
    body: String,
}

#[test]
fn test_ipv4_packet() {
    let packet = Ipv4Packet {
        version: 4,
        header_length: 20,
        type_of_service: 0,
        total_length: 60,
        identification: 12345,
        flags_and_fragment: 0x4000,
        ttl: 64,
        protocol: 6, // TCP
        checksum: 0x1234,
        source_address: vec![192, 168, 1, 100], // 4 bytes for IPv4
        dest_address: vec![8, 8, 8, 8],         // 4 bytes for IPv4
    };

    let bytes = packet.to_be_bytes();
    let (parsed, _) = Ipv4Packet::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(packet, parsed);

    // Verify IPv4 addresses are 4 bytes each
    assert_eq!(packet.source_address.len(), 4);
    assert_eq!(packet.dest_address.len(), 4);
}

#[test]
fn test_ipv6_packet() {
    let packet = Ipv6Packet {
        version: 6,
        traffic_class: 0,
        flow_label_high: 0,
        flow_label_low: 0,
        payload_length: 0,
        next_header: 6, // TCP
        hop_limit: 64,
        source_address: vec![
            0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70,
            0x73, 0x34,
        ], // 16 bytes for IPv6
        dest_address: vec![
            0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70,
            0x73, 0x35,
        ], // 16 bytes for IPv6
    };

    let bytes = packet.to_be_bytes();
    let (parsed, _) = Ipv6Packet::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(packet, parsed);

    // Verify IPv6 addresses are 16 bytes each
    assert_eq!(packet.source_address.len(), 16);
    assert_eq!(packet.dest_address.len(), 16);
}

#[test]
fn test_dns_message() {
    let message = DnsMessage {
        id: 0x1234,
        flags: 0x0100, // Standard query
        question_count: 2,
        answer_count: 1,
        authority_count: 0,
        additional_count: 0,
        questions: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10], // 2 * 5 = 10 bytes
        answers: vec![101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112], // 1 * 12 = 12 bytes
    };

    let bytes = message.to_be_bytes();
    let (parsed, _) = DnsMessage::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(message, parsed);
}

#[test]
fn test_mqtt_packet() {
    let packet = MqttPacket {
        fixed_header: 0x30, // PUBLISH packet
        remaining_length: 10,
        payload: vec![
            0, 5, b'h', b'e', b'l', b'l', b'o', b'w', b'o', b'r', b'l', b'd',
        ][..10]
            .to_vec(),
    };

    let bytes = packet.to_be_bytes();
    let (parsed, _) = MqttPacket::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(packet, parsed);
}

#[test]
fn test_tcp_segment() {
    let segment = TcpSegment {
        source_port: 80,
        dest_port: 8080,
        sequence_number: 0x12345678,
        ack_number: 0x87654321,
        data_offset_and_flags: 0x5018,
        window_size: 8192,
        checksum: 0xabcd,
        urgent_pointer: 0,
        options_length: 4,
        options: vec![0x02, 0x04, 0x05, 0xb4], // MSS option
        data_length: 11,
        data: vec![
            b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r', b'l', b'd',
        ],
    };

    let bytes = segment.to_be_bytes();
    let (parsed, _) = TcpSegment::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(segment, parsed);
}

#[test]
fn test_http_message() {
    // Create exactly 64 bytes of header data (2 * 32)
    let headers_str = "Content-Type: text/html\r\nConnection: close\r\n";
    let headers_padding = 64 - headers_str.len();
    let headers = format!("{}{}", headers_str, " ".repeat(headers_padding));

    let body_str = "<html><body>Hello</body></html>";

    let message = HttpMessage {
        status_code: 200,
        header_count: 2,
        headers,
        content_length: body_str.len() as u32,
        body: body_str.to_string(),
    };

    let bytes = message.to_be_bytes();
    let (parsed, _) = HttpMessage::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(message, parsed);
}

#[test]
fn test_protocol_serialization_consistency() {
    // Test that all protocols maintain consistency across endianness
    let mqtt = MqttPacket {
        fixed_header: 0x20,
        remaining_length: 5,
        payload: vec![1, 2, 3, 4, 5],
    };

    let be_bytes = mqtt.to_be_bytes();
    let le_bytes = mqtt.to_le_bytes();

    let (be_parsed, _) = MqttPacket::try_from_be_bytes(&be_bytes).unwrap();
    let (le_parsed, _) = MqttPacket::try_from_le_bytes(&le_bytes).unwrap();

    assert_eq!(mqtt, be_parsed);
    assert_eq!(mqtt, le_parsed);
}

#[test]
fn test_complex_field_dependencies() {
    // Test multiple dependent fields
    #[derive(BeBytes, Debug, PartialEq)]
    struct ComplexMessage {
        factor: u8,
        base: u8,
        #[With(size(factor * base))]
        primary_data: Vec<u8>,
        multiplier: u8,
        #[With(size((factor + base) * multiplier))]
        secondary_data: Vec<u8>,
    }

    let msg = ComplexMessage {
        factor: 3,
        base: 4,
        primary_data: vec![1; 12], // 3 * 4 = 12
        multiplier: 2,
        secondary_data: vec![2; 14], // (3 + 4) * 2 = 14
    };

    let bytes = msg.to_be_bytes();
    let (parsed, _) = ComplexMessage::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(msg, parsed);
}
