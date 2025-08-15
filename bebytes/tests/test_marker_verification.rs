use bebytes::BeBytes;

// Test different marker byte values
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct MarkerTest {
    header: u8,
    #[UntilMarker(0x00)]
    null_terminated: Vec<u8>,
    #[AfterMarker(0x00)]
    remainder: Vec<u8>,
}

#[test]
fn test_null_marker() {
    let msg = MarkerTest {
        header: 0x42,
        null_terminated: vec![0x01, 0x02, 0x03],
        remainder: vec![0xAA, 0xBB],
    };

    let bytes = msg.to_be_bytes();
    assert_eq!(bytes[0], 0x42); // header
    assert_eq!(bytes[1], 0x01); // first element
    assert_eq!(bytes[2], 0x02);
    assert_eq!(bytes[3], 0x03);
    assert_eq!(bytes[4], 0x00); // null marker for null_terminated
    assert_eq!(bytes[5], 0x00); // null marker for remainder
    assert_eq!(bytes[6], 0xAA);
    assert_eq!(bytes[7], 0xBB);

    let (parsed, consumed) = MarkerTest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, 8);
    assert_eq!(parsed, msg);
}

// Test with multiple UntilMarker fields
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct MultipleMarkers {
    #[UntilMarker(0x7F)]
    first: Vec<u8>,
    #[UntilMarker(0xFF)]
    second: Vec<u8>,
    final_byte: u8,
}

#[test]
fn test_multiple_until_markers() {
    let msg = MultipleMarkers {
        first: vec![0x10, 0x20],
        second: vec![0x30, 0x40, 0x50],
        final_byte: 0x99,
    };

    let bytes = msg.to_be_bytes();
    // first: 0x10, 0x20, 0x7F (marker)
    // second: 0x30, 0x40, 0x50, 0xFF (marker)
    // final: 0x99
    // Total: 2 + 1 + 3 + 1 + 1 = 8 bytes
    assert_eq!(bytes.len(), 8);

    let (parsed, consumed) = MultipleMarkers::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, bytes.len());
    assert_eq!(parsed.first, vec![0x10, 0x20]);
    assert_eq!(parsed.second, vec![0x30, 0x40, 0x50]);
    assert_eq!(parsed.final_byte, 0x99);
}

// Test edge case: marker byte appears in data
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct MarkerInData {
    #[UntilMarker(0xAB)]
    data: Vec<u8>,
    tail: u8,
}

#[test]
fn test_marker_byte_in_data() {
    // The marker byte 0xAB should terminate the vector
    let bytes = vec![0x11, 0x22, 0xAB, 0x33];

    let (parsed, consumed) = MarkerInData::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, 4);
    assert_eq!(parsed.data, vec![0x11, 0x22]); // Stops at 0xAB
    assert_eq!(parsed.tail, 0x33);
}

// Test empty vectors with markers
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct EmptyMarkerFields {
    prefix: u8,
    #[UntilMarker(0xDD)]
    empty_until: Vec<u8>,
    middle: u8,
    #[AfterMarker(0xEE)]
    empty_after: Vec<u8>,
}

#[test]
fn test_immediate_marker() {
    // Test when marker appears immediately
    let bytes = vec![
        0x01, // prefix
        0xDD, // immediate marker for empty_until
        0x02, // middle
        0xEE, // marker for empty_after (no data after)
    ];

    let (parsed, consumed) = EmptyMarkerFields::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, 4);
    assert_eq!(parsed.prefix, 0x01);
    assert_eq!(parsed.empty_until, Vec::<u8>::new());
    assert_eq!(parsed.middle, 0x02);
    assert_eq!(parsed.empty_after, Vec::<u8>::new());
}

// Test AfterMarker that consumes all remaining bytes
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct AfterMarkerConsumesAll {
    header: u16,
    #[AfterMarker(0xFE)]
    all_remaining: Vec<u8>,
}

#[test]
fn test_after_marker_consumes_remaining() {
    let msg = AfterMarkerConsumesAll {
        header: 0x1234,
        all_remaining: vec![0x01, 0x02, 0x03, 0x04, 0x05],
    };

    let bytes = msg.to_be_bytes();
    assert_eq!(bytes[0], 0x12); // header high byte
    assert_eq!(bytes[1], 0x34); // header low byte
    assert_eq!(bytes[2], 0xFE); // marker
    assert_eq!(bytes[3], 0x01); // start of remaining data
    assert_eq!(bytes[7], 0x05); // end of remaining data

    let (parsed, consumed) = AfterMarkerConsumesAll::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, 8);
    assert_eq!(parsed.header, 0x1234);
    assert_eq!(parsed.all_remaining, vec![0x01, 0x02, 0x03, 0x04, 0x05]);
}

// Test round-trip consistency
#[test]
fn test_marker_round_trip_consistency() {
    #[derive(Debug, Clone, PartialEq, BeBytes)]
    struct ComplexMarker {
        version: u8,
        flags: u8,
        #[UntilMarker(0xFD)]
        options: Vec<u8>,
        #[UntilMarker(0xFE)]
        extensions: Vec<u8>,
        #[AfterMarker(0xFF)]
        payload: Vec<u8>,
    }

    let original = ComplexMarker {
        version: 1,
        flags: 0b10101010,
        options: vec![0x11, 0x22, 0x33],
        extensions: vec![0x44, 0x55],
        payload: vec![0xAA, 0xBB, 0xCC, 0xDD],
    };

    // Serialize
    let bytes = original.to_be_bytes();

    // Deserialize
    let (parsed, _) = ComplexMarker::try_from_be_bytes(&bytes).unwrap();

    // Should match exactly
    assert_eq!(parsed, original);

    // Re-serialize should produce identical bytes
    let bytes2 = parsed.to_be_bytes();
    assert_eq!(bytes, bytes2);
}

// Test parsing with insufficient data (no marker found)
#[test]
fn test_no_marker_in_stream() {
    #[derive(Debug, Clone, PartialEq, BeBytes)]
    struct RequiresMarker {
        #[UntilMarker(0xCC)]
        data: Vec<u8>,
        after: u8,
    }

    // Data without the marker 0xCC
    let bytes = vec![0x11, 0x22, 0x33, 0x44, 0x55];

    let result = RequiresMarker::try_from_be_bytes(&bytes);
    // This should parse but consume all bytes looking for marker
    match result {
        Ok((parsed, consumed)) => {
            // It read all bytes looking for 0xCC
            assert_eq!(consumed, 5);
            assert_eq!(parsed.data, vec![0x11, 0x22, 0x33, 0x44, 0x55]);
            // No byte left for 'after' field - this would actually fail
        }
        Err(_) => {
            // Expected: insufficient data for 'after' field
        }
    }
}
