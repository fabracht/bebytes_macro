use bebytes::BeBytes;

// Test what happens when we try to use AfterMarker with Vec<Vec<u8>>
// This should produce a compile error
/* This should fail to compile:
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct InvalidAfterMarker {
    #[With(size(2))]
    #[AfterMarker(0xDD)]
    sections: Vec<Vec<u8>>,
}
*/

// Test missing size attribute - should produce compile error
/* This should fail to compile:
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct MissingSizeAttribute {
    #[UntilMarker(0xFF)]
    segments: Vec<Vec<u8>>,
}
*/

// Test edge case: insufficient data during parsing
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct InsufficientDataTest {
    count: u8,
    #[FromField(count)]
    #[UntilMarker(0xAA)]
    segments: Vec<Vec<u8>>,
}

#[test]
fn test_insufficient_data() {
    // We expect 3 segments but only provide data for 2
    let bytes = vec![
        3, // count = 3 segments expected
        0x01, 0x02, 0xAA, // segment 1 + marker
        0x03, 0x04,
        0xAA, // segment 2 + marker
              // Missing segment 3 data - should result in empty third segment
    ];

    let result = InsufficientDataTest::try_from_be_bytes(&bytes);

    // Should fail with MarkerNotFound for third segment (no marker at end)
    match result {
        Err(bebytes::BeBytesError::MarkerNotFound { marker, field }) => {
            assert_eq!(marker, 0xAA);
            assert_eq!(field, "segments");
        }
        Ok(_) => panic!("Expected MarkerNotFound error"),
        Err(e) => panic!("Wrong error type: {:?}", e),
    }
}

// Test missing markers - segments without terminal markers
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct MissingMarkersTest {
    count: u8,
    #[FromField(count)]
    #[UntilMarker(0xBB)]
    segments: Vec<Vec<u8>>,
}

#[test]
fn test_missing_markers() {
    // Expect 2 segments, but no markers in data
    let bytes = vec![
        2, // count = 2 segments expected
        0x11, 0x22, 0x33, 0x44, // Data with no 0xBB markers
    ];

    let result = MissingMarkersTest::try_from_be_bytes(&bytes);

    // Should fail with MarkerNotFound for first segment (no markers found)
    match result {
        Err(bebytes::BeBytesError::MarkerNotFound { marker, field }) => {
            assert_eq!(marker, 0xBB);
            assert_eq!(field, "segments");
        }
        Ok(_) => panic!("Expected MarkerNotFound error"),
        Err(e) => panic!("Wrong error type: {:?}", e),
    }
}

// Test zero-sized segments count
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct ZeroSegmentsTest {
    count: u8,
    #[FromField(count)]
    #[UntilMarker(0xCC)]
    segments: Vec<Vec<u8>>,
    footer: u8,
}

#[test]
fn test_zero_segments() {
    let msg = ZeroSegmentsTest {
        count: 0,
        segments: vec![],
        footer: 0x99,
    };

    let bytes = msg.to_be_bytes();

    // Expected structure:
    // count: 0
    // footer: 0x99 (no segments to write)
    let expected = vec![0, 0x99];

    assert_eq!(bytes, expected);

    let (parsed, consumed) = ZeroSegmentsTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, bytes.len());
    assert_eq!(parsed, msg);
    assert_eq!(parsed.segments.len(), 0);
}

// Test all empty segments
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct AllEmptySegments {
    #[With(size(3))]
    #[UntilMarker(0xEE)]
    segments: Vec<Vec<u8>>,
}

#[test]
fn test_all_empty_segments() {
    let msg = AllEmptySegments {
        segments: vec![vec![], vec![], vec![]],
    };

    let bytes = msg.to_be_bytes();

    // Expected structure: just 3 markers
    let expected = vec![0xEE, 0xEE, 0xEE];

    assert_eq!(bytes, expected);

    let (parsed, consumed) = AllEmptySegments::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, bytes.len());
    assert_eq!(parsed, msg);
    assert_eq!(parsed.segments.len(), 3);
    assert!(parsed.segments.iter().all(|seg| seg.is_empty()));
}

// Test marker byte appearing in data
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct MarkerInDataTest {
    #[With(size(2))]
    #[UntilMarker(0xFF)]
    segments: Vec<Vec<u8>>,
}

#[test]
fn test_marker_in_data_terminates_segment() {
    // Test that marker byte correctly terminates segments
    let bytes = vec![
        0x01, 0xFF, // First segment: [0x01] + marker
        0x02, 0xFF, // Second segment: [0x02] + marker
        0x33, // Extra data that won't be consumed (only 2 segments expected)
    ];

    let (parsed, consumed) = MarkerInDataTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, 4); // Only consumed first 4 bytes (2 segments + markers)
    assert_eq!(parsed.segments.len(), 2);
    assert_eq!(parsed.segments[0], vec![0x01]);
    assert_eq!(parsed.segments[1], vec![0x02]);
}

// Test large segment counts
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct LargeSegmentCount {
    count: u8,
    #[FromField(count)]
    #[UntilMarker(0x00)]
    segments: Vec<Vec<u8>>,
}

#[test]
fn test_large_segment_count() {
    // Test with maximum u8 value
    let mut segments = Vec::new();
    let mut expected_bytes = vec![255]; // count = 255

    // Create 255 single-byte segments
    for i in 1..=255 {
        segments.push(vec![i as u8]);
        expected_bytes.push(i as u8);
        expected_bytes.push(0x00); // marker
    }

    let msg = LargeSegmentCount {
        count: 255,
        segments,
    };

    let bytes = msg.to_be_bytes();
    assert_eq!(bytes, expected_bytes);
    assert_eq!(bytes.len(), 1 + 255 * 2); // count + (255 segments * 2 bytes each)

    let (parsed, consumed) = LargeSegmentCount::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, bytes.len());
    assert_eq!(parsed, msg);
    assert_eq!(parsed.segments.len(), 255);
}
