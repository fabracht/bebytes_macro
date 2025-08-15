use bebytes::BeBytes;

// Test basic Vec<Vec<u8>> with UntilMarker and fixed size
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct FixedSizeSegments {
    header: u8,
    #[With(size(3))]
    #[UntilMarker(0xFF)]
    segments: Vec<Vec<u8>>,
    footer: u8,
}

#[test]
fn test_fixed_size_segments() {
    let msg = FixedSizeSegments {
        header: 0x42,
        segments: vec![
            vec![0x01, 0x02],       // First segment
            vec![0x03, 0x04, 0x05], // Second segment
            vec![0x06],             // Third segment
        ],
        footer: 0x99,
    };

    let bytes = msg.to_be_bytes();

    // Expected structure:
    // header: 0x42
    // segment1: 0x01, 0x02, 0xFF (marker)
    // segment2: 0x03, 0x04, 0x05, 0xFF (marker)
    // segment3: 0x06, 0xFF (marker)
    // footer: 0x99
    let expected = vec![
        0x42, // header
        0x01, 0x02, 0xFF, // segment 1 + marker
        0x03, 0x04, 0x05, 0xFF, // segment 2 + marker
        0x06, 0xFF, // segment 3 + marker
        0x99, // footer
    ];

    assert_eq!(bytes, expected);

    let (parsed, consumed) = FixedSizeSegments::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, bytes.len());
    assert_eq!(parsed, msg);
}

// Test Vec<Vec<u8>> with FromField size control
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct VariableSizeSegments {
    version: u8,
    segment_count: u8,
    #[FromField(segment_count)]
    #[UntilMarker(0xAA)]
    data_segments: Vec<Vec<u8>>,
    checksum: u16,
}

#[test]
fn test_variable_size_segments() {
    let msg = VariableSizeSegments {
        version: 1,
        segment_count: 2,
        data_segments: vec![
            vec![0x11, 0x22, 0x33], // First segment
            vec![0x44, 0x55],       // Second segment
        ],
        checksum: 0x1234,
    };

    let bytes = msg.to_be_bytes();

    // Expected structure:
    // version: 1
    // segment_count: 2
    // segment1: 0x11, 0x22, 0x33, 0xAA (marker)
    // segment2: 0x44, 0x55, 0xAA (marker)
    // checksum: 0x12, 0x34
    let expected = vec![
        1, // version
        2, // segment_count
        0x11, 0x22, 0x33, 0xAA, // segment 1 + marker
        0x44, 0x55, 0xAA, // segment 2 + marker
        0x12, 0x34, // checksum (big-endian)
    ];

    assert_eq!(bytes, expected);

    let (parsed, consumed) = VariableSizeSegments::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, bytes.len());
    assert_eq!(parsed, msg);
}

// Test empty segments
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct EmptySegments {
    count: u8,
    #[FromField(count)]
    #[UntilMarker(0x00)]
    segments: Vec<Vec<u8>>,
}

#[test]
fn test_empty_segments() {
    let msg = EmptySegments {
        count: 3,
        segments: vec![
            vec![0x01],       // Non-empty segment
            vec![],           // Empty segment
            vec![0x02, 0x03], // Non-empty segment
        ],
    };

    let bytes = msg.to_be_bytes();

    // Expected structure:
    // count: 3
    // segment1: 0x01, 0x00 (marker)
    // segment2: 0x00 (marker only - empty segment)
    // segment3: 0x02, 0x03, 0x00 (marker)
    let expected = vec![
        3, // count
        0x01, 0x00, // segment 1 + marker
        0x00, // empty segment (just marker)
        0x02, 0x03, 0x00, // segment 3 + marker
    ];

    assert_eq!(bytes, expected);

    let (parsed, consumed) = EmptySegments::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, bytes.len());
    assert_eq!(parsed, msg);
}

// Test round-trip consistency with complex protocol
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct ComplexProtocol {
    magic: u32,
    version: u8,
    option_count: u8,
    #[FromField(option_count)]
    #[UntilMarker(0xFE)]
    options: Vec<Vec<u8>>,
    payload_len: u16,
    #[FromField(payload_len)]
    payload: Vec<u8>,
}

#[test]
fn test_complex_protocol_round_trip() {
    let original = ComplexProtocol {
        magic: 0x12345678,
        version: 2,
        option_count: 3,
        options: vec![
            vec![0x01, 0x02],       // Option 1
            vec![0x03, 0x04, 0x05], // Option 2
            vec![0x06],             // Option 3
        ],
        payload_len: 4,
        payload: vec![0xAA, 0xBB, 0xCC, 0xDD],
    };

    // Serialize
    let bytes = original.to_be_bytes();

    // Deserialize
    let (parsed, consumed) = ComplexProtocol::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, bytes.len());
    assert_eq!(parsed, original);

    // Re-serialize should produce identical bytes
    let bytes2 = parsed.to_be_bytes();
    assert_eq!(bytes, bytes2);
}
