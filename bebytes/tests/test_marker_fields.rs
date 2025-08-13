use bebytes::BeBytes;

#[derive(Debug, Clone, PartialEq, BeBytes)]
struct CoapMessage {
    pub header: u8,
    pub token_length: u8,
    #[UntilMarker(0xFF)]
    pub options: Vec<u8>,
    #[AfterMarker(0xFF)]
    pub payload: Vec<u8>,
}

#[test]
fn test_until_marker_field() {
    let msg = CoapMessage {
        header: 0x45,
        token_length: 4,
        options: vec![0x11, 0x22, 0x33],
        payload: vec![0xAA, 0xBB, 0xCC],
    };

    let bytes = msg.to_be_bytes();
    
    // Should be: header(1) + token_length(1) + options(3) + marker(1) + marker(1) + payload(3) = 10 bytes
    assert_eq!(bytes.len(), 10);
    assert_eq!(bytes[0], 0x45); // header
    assert_eq!(bytes[1], 4);    // token_length
    assert_eq!(bytes[2], 0x11); // first option
    assert_eq!(bytes[3], 0x22); // second option
    assert_eq!(bytes[4], 0x33); // third option
    assert_eq!(bytes[5], 0xFF); // marker for options
    assert_eq!(bytes[6], 0xFF); // marker for payload
    assert_eq!(bytes[7], 0xAA); // first payload byte
    assert_eq!(bytes[8], 0xBB); // second payload byte
    assert_eq!(bytes[9], 0xCC); // third payload byte

    // Test parsing
    let (parsed, consumed) = CoapMessage::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, 10);
    assert_eq!(parsed.header, 0x45);
    assert_eq!(parsed.token_length, 4);
    assert_eq!(parsed.options, vec![0x11, 0x22, 0x33]);
    assert_eq!(parsed.payload, vec![0xAA, 0xBB, 0xCC]);
}

#[test]
fn test_empty_marker_fields() {
    let msg = CoapMessage {
        header: 0x40,
        token_length: 0,
        options: vec![],
        payload: vec![],
    };

    let bytes = msg.to_be_bytes();
    
    // Should be: header(1) + token_length(1) + marker(1) + marker(1) = 4 bytes
    assert_eq!(bytes.len(), 4);
    assert_eq!(bytes[0], 0x40); // header
    assert_eq!(bytes[1], 0);    // token_length
    assert_eq!(bytes[2], 0xFF); // marker for options
    assert_eq!(bytes[3], 0xFF); // marker for payload

    // Test parsing
    let (parsed, consumed) = CoapMessage::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(consumed, 4);
    assert_eq!(parsed.header, 0x40);
    assert_eq!(parsed.token_length, 0);
    assert_eq!(parsed.options, Vec::<u8>::new());
    assert_eq!(parsed.payload, Vec::<u8>::new());
}

#[test]
fn test_after_marker_without_marker() {
    // Test case where AfterMarker doesn't find the marker
    let bytes = vec![0x40, 0x00]; // Just header and token_length, no markers
    
    let result = CoapMessage::try_from_be_bytes(&bytes);
    // This should still parse but the payload will be empty
    match result {
        Ok((msg, consumed)) => {
            assert_eq!(consumed, 2);
            assert_eq!(msg.header, 0x40);
            assert_eq!(msg.token_length, 0);
            assert_eq!(msg.options, Vec::<u8>::new());
            assert_eq!(msg.payload, Vec::<u8>::new());
        }
        Err(_) => {
            // Also acceptable if it errors due to missing marker
        }
    }
}