use bebytes::BeBytes;

#[test]
fn test_marker_not_found_with_char_literal() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Protocol {
        version: u8,
        #[UntilMarker('\n')]
        command: Vec<u8>,
        checksum: u32,
    }

    // No newline marker in data
    let bytes = vec![
        0x01, // version
        b'H', b'E', b'L', b'L', b'O', // "HELLO" without newline
        0x12, 0x34, 0x56, 0x78, // these bytes will be consumed as part of command
    ];

    match Protocol::try_from_be_bytes(&bytes) {
        Ok(_) => panic!("Should have failed with MarkerNotFound"),
        Err(bebytes::BeBytesError::MarkerNotFound { marker, field }) => {
            assert_eq!(marker, b'\n');
            assert_eq!(field, "command");
        }
        Err(e) => panic!("Wrong error type: {:?}", e),
    }
}

#[test]
fn test_multiple_fields_after_missing_marker() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Complex {
        header: u16,
        #[UntilMarker(0xFF)]
        data1: Vec<u8>,
        middle: u32,
        #[UntilMarker(0xFE)]
        data2: Vec<u8>,
        footer: u16,
    }

    // First marker missing
    let bytes = vec![
        0x12, 0x34, // header
        0xAA, 0xBB, 0xCC, 0xDD, // data1 (no 0xFF marker)
        0x11, 0x22, 0x33, 0x44, // would be middle
        0xFE, // marker for data2
        0x99, 0x88, // would be footer
    ];

    match Complex::try_from_be_bytes(&bytes) {
        Ok(_) => panic!("Should have failed"),
        Err(bebytes::BeBytesError::MarkerNotFound { marker, field }) => {
            assert_eq!(marker, 0xFF);
            assert_eq!(field, "data1");
        }
        Err(e) => panic!("Wrong error: {:?}", e),
    }
}

#[test]
fn test_nested_vec_missing_marker() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Nested {
        segment_count: u8,
        #[FromField(segment_count)]
        #[UntilMarker(0x00)]
        segments: Vec<Vec<u8>>,
        crc: u16,
    }

    let bytes = vec![
        0x02, // segment_count = 2
        0x41, 0x42, 0x00, // segment 1: "AB" with null terminator
        0x43, 0x44,
        0x45, // segment 2: "CDE" without null terminator
              // Missing bytes for CRC
    ];

    match Nested::try_from_be_bytes(&bytes) {
        Ok(_) => panic!("Should have failed"),
        Err(bebytes::BeBytesError::MarkerNotFound { marker, field }) => {
            assert_eq!(marker, 0x00);
            assert_eq!(field, "segments");
        }
        Err(e) => panic!("Wrong error: {:?}", e),
    }
}

#[test]
fn test_marker_found_continues_parsing() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Good {
        id: u8,
        #[UntilMarker(0x7E)]
        name: Vec<u8>,
        value: u16,
    }

    let bytes = vec![
        0x42, // id
        0x41, 0x42, 0x43, 0x7E, // "ABC" with marker
        0x12, 0x34, // value
    ];

    let (parsed, consumed) = Good::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.id, 0x42);
    assert_eq!(parsed.name, vec![0x41, 0x42, 0x43]);
    assert_eq!(parsed.value, 0x1234);
    assert_eq!(consumed, 7); // 1 (id) + 3 (name) + 1 (marker) + 2 (value) = 7
}

#[test]
fn test_consecutive_marker_fields_missing_first() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Double {
        #[UntilMarker(0xAA)]
        first: Vec<u8>,
        #[UntilMarker(0xBB)]
        second: Vec<u8>,
        tail: u8,
    }

    let bytes = vec![
        0x11, 0x22, 0x33, // first field data (no 0xAA)
        0xBB, // marker for second field
        0x99, // tail
    ];

    match Double::try_from_be_bytes(&bytes) {
        Ok(_) => panic!("Should have failed"),
        Err(bebytes::BeBytesError::MarkerNotFound { marker, field }) => {
            assert_eq!(marker, 0xAA);
            assert_eq!(field, "first");
        }
        Err(e) => panic!("Wrong error: {:?}", e),
    }
}

#[test]
fn test_consecutive_marker_fields_missing_second() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Double {
        #[UntilMarker(0xAA)]
        first: Vec<u8>,
        #[UntilMarker(0xBB)]
        second: Vec<u8>,
        tail: u8,
    }

    let bytes = vec![
        0x11, 0x22, 0xAA, // first field with marker
        0x33, 0x44, 0x55, // second field data (no 0xBB)
        0x99, // would be consumed as part of second
    ];

    match Double::try_from_be_bytes(&bytes) {
        Ok(_) => panic!("Should have failed"),
        Err(bebytes::BeBytesError::MarkerNotFound { marker, field }) => {
            assert_eq!(marker, 0xBB);
            assert_eq!(field, "second");
        }
        Err(e) => panic!("Wrong error: {:?}", e),
    }
}

#[test]
fn test_tab_marker_not_found() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct TabDelimited {
        header: u8,
        #[UntilMarker('\t')]
        field1: Vec<u8>,
        #[UntilMarker('\t')]
        field2: Vec<u8>,
        value: u16,
    }

    let bytes = vec![
        0x01, // header
        b'A', b'B', b'C', // field1 (no tab)
        b'D', b'E', b'F', // field2 (no tab)
        0x12, 0x34, // value
    ];

    match TabDelimited::try_from_be_bytes(&bytes) {
        Ok(_) => panic!("Should have failed"),
        Err(bebytes::BeBytesError::MarkerNotFound { marker, field }) => {
            assert_eq!(marker, b'\t');
            assert_eq!(field, "field1");
        }
        Err(e) => panic!("Wrong error: {:?}", e),
    }
}

#[test]
fn test_empty_buffer_before_marker_field() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Empty {
        id: u32,
        #[UntilMarker(0xFF)]
        data: Vec<u8>,
        tail: u8,
    }

    let bytes = vec![
        0x12, 0x34, 0x56, 0x78, // id only
    ];

    match Empty::try_from_be_bytes(&bytes) {
        Ok(_) => panic!("Should have failed"),
        Err(bebytes::BeBytesError::MarkerNotFound { marker, field }) => {
            assert_eq!(marker, 0xFF);
            assert_eq!(field, "data");
        }
        Err(e) => panic!("Wrong error: {:?}", e),
    }
}

#[test]
fn test_all_segments_missing_markers() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct AllMissing {
        count: u8,
        #[FromField(count)]
        #[UntilMarker(0x00)]
        items: Vec<Vec<u8>>,
        checksum: u8,
    }

    let bytes = vec![
        0x03, // count = 3
        0x41, 0x42, 0x43, // All data without any null terminators
        0x44, 0x45, // More data
        0xFF, // checksum would be here
    ];

    match AllMissing::try_from_be_bytes(&bytes) {
        Ok(_) => panic!("Should have failed on first segment"),
        Err(bebytes::BeBytesError::MarkerNotFound { marker, field }) => {
            assert_eq!(marker, 0x00);
            assert_eq!(field, "items");
        }
        Err(e) => panic!("Wrong error: {:?}", e),
    }
}

#[test]
fn test_complex_protocol_simulation() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Message {
        msg_type: u8,
        sender_id: u16,
        #[UntilMarker(0x03)] // ETX (End of Text)
        payload: Vec<u8>,
        timestamp: u32,
        #[UntilMarker(0x04)] // EOT (End of Transmission)
        metadata: Vec<u8>,
        crc: u16,
    }

    // Missing first marker (ETX)
    let bytes = vec![
        0x01, // msg_type
        0x12, 0x34, // sender_id
        b'H', b'e', b'l', b'l', b'o', // payload without ETX
        0x56, 0x78, 0x9A, 0xBC, // consumed as payload
        0x04, // EOT marker
        0xAB, 0xCD, // would be crc
    ];

    match Message::try_from_be_bytes(&bytes) {
        Ok(_) => panic!("Should have failed"),
        Err(bebytes::BeBytesError::MarkerNotFound { marker, field }) => {
            assert_eq!(marker, 0x03);
            assert_eq!(field, "payload");
        }
        Err(e) => panic!("Wrong error: {:?}", e),
    }

    // Correct message with all markers
    let bytes_correct = vec![
        0x01, // msg_type
        0x12, 0x34, // sender_id
        b'H', b'i', 0x03, // payload with ETX
        0x56, 0x78, 0x9A, 0xBC, // timestamp
        b'M', b'D', 0x04, // metadata with EOT
        0xAB, 0xCD, // crc
    ];

    let (parsed, consumed) = Message::try_from_be_bytes(&bytes_correct).unwrap();
    assert_eq!(parsed.msg_type, 0x01);
    assert_eq!(parsed.sender_id, 0x1234);
    assert_eq!(parsed.payload, vec![b'H', b'i']);
    assert_eq!(parsed.timestamp, 0x56789ABC);
    assert_eq!(parsed.metadata, vec![b'M', b'D']);
    assert_eq!(parsed.crc, 0xABCD);
    assert_eq!(consumed, 15);
}
