use bebytes::BeBytes;

#[test]
fn test_until_marker_not_found() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Protocol {
        header: u8,
        #[UntilMarker(0xFF)]
        data: Vec<u8>,
        footer: u16,
    }

    // Case 1: Marker exists - normal behavior
    let bytes_with_marker = vec![
        0x01, // header
        0xAA, 0xBB, // data
        0xFF, // marker
        0x12, 0x34, // footer
    ];

    let (parsed, consumed) = Protocol::try_from_be_bytes(&bytes_with_marker).unwrap();
    assert_eq!(parsed.header, 0x01);
    assert_eq!(parsed.data, vec![0xAA, 0xBB]);
    assert_eq!(parsed.footer, 0x1234);
    assert_eq!(consumed, 6);

    // Case 2: No marker found - should get MarkerNotFound error
    let bytes_no_marker = vec![
        0x01, // header
        0xAA, 0xBB, // will be consumed as data
        0x12, 0x34, // will also be consumed as data (no footer!)
    ];

    match Protocol::try_from_be_bytes(&bytes_no_marker) {
        Ok((parsed, consumed)) => {
            panic!(
                "Should fail but got: data={:?}, footer={:04X}, consumed={}",
                parsed.data, parsed.footer, consumed
            );
        }
        Err(bebytes::BeBytesError::MarkerNotFound { marker, field }) => {
            assert_eq!(marker, 0xFF);
            assert_eq!(field, "data");
        }
        Err(e) => {
            panic!("Expected MarkerNotFound error but got: {:?}", e);
        }
    }
}

#[test]
fn test_after_marker_not_found() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Message {
        header: u32,
        #[AfterMarker(0xFF)]
        payload: Vec<u8>,
    }

    // Case 1: Marker exists - normal behavior
    let bytes_with_marker = vec![
        0x12, 0x34, 0x56, 0x78, // header
        0xAA, 0xBB, // skip these
        0xFF, // marker
        0xCC, 0xDD, 0xEE, // payload
    ];

    let (parsed, consumed) = Message::try_from_be_bytes(&bytes_with_marker).unwrap();
    assert_eq!(parsed.header, 0x12345678);
    assert_eq!(parsed.payload, vec![0xCC, 0xDD, 0xEE]);
    assert_eq!(consumed, 10);

    // Case 2: No marker found - field becomes empty
    let bytes_no_marker = vec![
        0x12, 0x34, 0x56, 0x78, // header
        0xAA, 0xBB, 0xCC, // no marker, so these aren't consumed
    ];

    let (parsed, consumed) = Message::try_from_be_bytes(&bytes_no_marker).unwrap();
    assert_eq!(parsed.header, 0x12345678);
    assert_eq!(parsed.payload, vec![]); // Empty because no marker found
    assert_eq!(consumed, 4); // Only header consumed
}

#[test]
fn test_multiple_markers_missing_some() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct MultiSection {
        count: u8,
        #[FromField(count)]
        #[UntilMarker(0xFF)]
        sections: Vec<Vec<u8>>,
        footer: u8,
    }

    // Case: Some sections have markers, some don't
    let bytes = vec![
        0x03, // count = 3
        0xAA, 0xBB, 0xFF, // section 1 with marker
        0xCC, 0xFF, // section 2 with marker
        0xDD,
        0xEE, // section 3 without marker - should error
              // No bytes left for footer!
    ];

    match MultiSection::try_from_be_bytes(&bytes) {
        Ok((parsed, consumed)) => {
            panic!(
                "Should fail but got: sections={:?}, footer={}, consumed={}",
                parsed.sections, parsed.footer, consumed
            );
        }
        Err(bebytes::BeBytesError::MarkerNotFound { marker, field }) => {
            assert_eq!(marker, 0xFF);
            assert_eq!(field, "sections");
        }
        Err(e) => {
            panic!("Expected MarkerNotFound error but got: {:?}", e);
        }
    }
}

#[test]
fn test_edge_case_immediate_marker() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Edge {
        id: u8,
        #[UntilMarker(0xFF)]
        data: Vec<u8>,
        tail: u8,
    }

    // Marker appears immediately
    let bytes = vec![
        0x42, // id
        0xFF, // immediate marker
        0x99, // tail
    ];

    let (parsed, consumed) = Edge::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.id, 0x42);
    assert_eq!(parsed.data, vec![]); // Empty data
    assert_eq!(parsed.tail, 0x99);
    assert_eq!(consumed, 3);
}

#[test]
fn test_marker_at_end_causes_insufficient_data() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Protocol {
        header: u8,
        #[UntilMarker(0xFF)]
        data: Vec<u8>,
        footer: u16, // Requires 2 bytes
    }

    // Marker at the very end, no bytes for footer
    let bytes = vec![
        0x01, // header
        0xAA, 0xBB, // data
        0xFF, // marker at end
    ];

    match Protocol::try_from_be_bytes(&bytes) {
        Ok((parsed, _)) => {
            panic!("Should fail but got: {:?}", parsed);
        }
        Err(e) => {
            assert!(matches!(
                e,
                bebytes::BeBytesError::InsufficientData {
                    expected: 2,
                    actual: 0
                }
            ));
        }
    }
}

#[test]
fn test_until_marker_last_field_no_error() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct LastFieldProtocol {
        header: u32,
        #[UntilMarker(0xFF)]
        data: Vec<u8>, // Last field
    }

    // No marker, but it's the last field so it should consume all remaining bytes
    let bytes = vec![
        0x12, 0x34, 0x56, 0x78, // header
        0xAA, 0xBB, 0xCC, 0xDD, // data (no marker)
    ];

    let (parsed, consumed) = LastFieldProtocol::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.header, 0x12345678);
    assert_eq!(parsed.data, vec![0xAA, 0xBB, 0xCC, 0xDD]);
    assert_eq!(consumed, 8);
}

#[test]
fn test_display_for_marker_not_found() {
    let err = bebytes::BeBytesError::MarkerNotFound {
        marker: 0xFF,
        field: "test_field",
    };

    let display = format!("{}", err);
    assert_eq!(display, "Marker byte 0xFF not found in field 'test_field'");
}
