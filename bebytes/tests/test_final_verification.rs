use bebytes::BeBytes;

// Real-world CoAP-like protocol test
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct CoapMessage {
    // CoAP header
    version: u8,
    msg_type: u8,
    token_length: u8,
    code: u8,
    message_id: u16,

    // Token (variable length based on token_length)
    #[FromField(token_length)]
    token: Vec<u8>,

    // Options count and options (each terminated by 0xFF)
    option_count: u8,
    #[FromField(option_count)]
    #[UntilMarker(0xFF)]
    options: Vec<Vec<u8>>,

    // Payload marker and payload
    payload_marker: u8, // Should be 0xFF to indicate payload follows
    payload: Vec<u8>,   // Remaining bytes
}

#[test]
fn test_coap_like_protocol() {
    let msg = CoapMessage {
        version: 1,
        msg_type: 0,
        token_length: 4,
        code: 69, // 2.05 Content
        message_id: 0x1234,
        token: vec![0xAA, 0xBB, 0xCC, 0xDD],
        option_count: 3,
        options: vec![
            vec![0x11, 0x22],       // Option 1
            vec![0x33, 0x44, 0x55], // Option 2
            vec![0x66],             // Option 3
        ],
        payload_marker: 0xFF,
        payload: vec![0x48, 0x65, 0x6C, 0x6C, 0x6F], // "Hello"
    };

    let bytes = msg.to_be_bytes();
    println!("Serialized {} bytes", bytes.len());

    let (parsed, consumed) = CoapMessage::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, bytes.len());
    assert_eq!(parsed, msg);

    // Verify round-trip
    let bytes2 = parsed.to_be_bytes();
    assert_eq!(bytes, bytes2);
}

// Test that shows the old way vs new way
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct OldWayProtocol {
    count: u8,
    #[UntilMarker(0xAA)]
    section1: Vec<u8>,
    #[UntilMarker(0xAA)]
    section2: Vec<u8>,
    #[UntilMarker(0xAA)]
    section3: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, BeBytes)]
struct NewWayProtocol {
    count: u8,
    #[FromField(count)]
    #[UntilMarker(0xAA)]
    sections: Vec<Vec<u8>>,
}

#[test]
fn test_old_vs_new_pattern() {
    // Old way - fixed 3 sections
    let old = OldWayProtocol {
        count: 3,
        section1: vec![0x01, 0x02],
        section2: vec![0x03, 0x04],
        section3: vec![0x05, 0x06],
    };

    let old_bytes = old.to_be_bytes();

    // New way - dynamic number of sections
    let new = NewWayProtocol {
        count: 3,
        sections: vec![vec![0x01, 0x02], vec![0x03, 0x04], vec![0x05, 0x06]],
    };

    let new_bytes = new.to_be_bytes();

    // Both should produce identical output
    assert_eq!(old_bytes, new_bytes);

    // New way can handle variable counts
    let new_variable = NewWayProtocol {
        count: 5,
        sections: vec![
            vec![0x01],
            vec![0x02, 0x03],
            vec![0x04],
            vec![0x05, 0x06, 0x07],
            vec![0x08],
        ],
    };

    let var_bytes = new_variable.to_be_bytes();
    let (parsed, _) = NewWayProtocol::try_from_be_bytes(&var_bytes).unwrap();
    assert_eq!(parsed, new_variable);
}

// Verify compile-time validation
#[test]
fn test_compile_time_checks_enforced() {
    // This should compile fine
    #[derive(Debug, Clone, PartialEq, BeBytes)]
    struct ValidStruct {
        #[With(size(2))]
        #[UntilMarker(0xBB)]
        segments: Vec<Vec<u8>>,
    }

    let valid = ValidStruct {
        segments: vec![vec![1, 2], vec![3, 4]],
    };

    let bytes = valid.to_be_bytes();
    let (parsed, _) = ValidStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, valid);

    // The following would not compile (commented out):
    /*
    #[derive(Debug, Clone, PartialEq, BeBytes)]
    struct InvalidNoSize {
        #[UntilMarker(0xCC)]
        segments: Vec<Vec<u8>>,  // ERROR: Missing size control
    }

    #[derive(Debug, Clone, PartialEq, BeBytes)]
    struct InvalidAfterMarker {
        #[With(size(2))]
        #[AfterMarker(0xDD)]
        segments: Vec<Vec<u8>>,  // ERROR: AfterMarker not supported
    }
    */
}

// Performance test - ensure no regression
#[test]
fn test_performance_characteristics() {
    use std::time::Instant;

    #[derive(Debug, Clone, PartialEq, BeBytes)]
    struct LargeMessage {
        segment_count: u8,
        #[FromField(segment_count)]
        #[UntilMarker(0x00)]
        segments: Vec<Vec<u8>>,
    }

    // Create a message with many segments
    // Note: avoid using 0x00 in data since it's the marker
    let mut segments = Vec::new();
    for i in 0..100 {
        // Use i+1 to avoid 0x00 which is the marker
        segments.push(vec![(i + 1) as u8; (i % 10 + 1) as usize]);
    }

    let msg = LargeMessage {
        segment_count: 100,
        segments,
    };

    // Measure serialization
    let start = Instant::now();
    let bytes = msg.to_be_bytes();
    let serialize_time = start.elapsed();

    // Measure deserialization
    let start = Instant::now();
    let (parsed, _) = LargeMessage::try_from_be_bytes(&bytes).unwrap();
    let deserialize_time = start.elapsed();

    println!("Serialization time: {:?}", serialize_time);
    println!("Deserialization time: {:?}", deserialize_time);

    assert_eq!(parsed, msg);

    // Ensure times are reasonable (< 1ms for 100 segments)
    assert!(serialize_time.as_millis() < 1);
    assert!(deserialize_time.as_millis() < 1);
}

// Test integration with existing BeBytes features
#[derive(Debug, Clone, PartialEq, BeBytes)]
struct IntegratedFeatures {
    #[bits(4)]
    version: u8,
    #[bits(4)]
    flags: u8,

    name_len: u8,
    #[FromField(name_len)]
    name: String,

    segment_count: u8,
    #[FromField(segment_count)]
    #[UntilMarker(0xEE)]
    data_segments: Vec<Vec<u8>>,

    checksum: u32,
}

#[test]
fn test_integration_with_other_features() {
    let msg = IntegratedFeatures {
        version: 2,
        flags: 0b1010,
        name_len: 5,
        name: "Hello".to_string(),
        segment_count: 2,
        data_segments: vec![vec![0x11, 0x22], vec![0x33, 0x44, 0x55]],
        checksum: 0x12345678,
    };

    let bytes = msg.to_be_bytes();
    let (parsed, consumed) = IntegratedFeatures::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(consumed, bytes.len());
    assert_eq!(parsed.version, 2);
    assert_eq!(parsed.flags, 0b1010);
    assert_eq!(parsed.name, "Hello");
    assert_eq!(parsed.data_segments.len(), 2);
    assert_eq!(parsed.checksum, 0x12345678);

    // Full equality check
    assert_eq!(parsed, msg);
}
