use bebytes::BeBytes;

#[test]
fn test_until_marker_with_newline_char() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct LineProtocol {
        header: u8,
        #[UntilMarker('\n')]
        line: Vec<u8>,
        footer: u16,
    }

    let msg = LineProtocol {
        header: 0x42,
        line: b"Hello World".to_vec(),
        footer: 0x1234,
    };

    let bytes = msg.to_be_bytes();
    assert_eq!(bytes[0], 0x42);
    assert_eq!(&bytes[1..12], b"Hello World");
    assert_eq!(bytes[12], b'\n');
    assert_eq!(bytes[13], 0x12);
    assert_eq!(bytes[14], 0x34);

    let (parsed, _) = LineProtocol::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_until_marker_with_null_char() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct CString {
        id: u32,
        #[UntilMarker('\0')]
        text: Vec<u8>,
        checksum: u8,
    }

    let msg = CString {
        id: 0xDEADBEEF,
        text: b"null-terminated".to_vec(),
        checksum: 0xFF,
    };

    let bytes = msg.to_be_bytes();
    let (parsed, _) = CString::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, msg);

    // Check null terminator is present
    assert_eq!(bytes[4 + msg.text.len()], 0x00);
}

#[test]
fn test_after_marker_with_tab_char() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct TabDelimited {
        version: u8,
        #[AfterMarker('\t')]
        content: Vec<u8>,
    }

    let _msg = TabDelimited {
        version: 1,
        content: b"after tab".to_vec(),
    };

    let mut bytes = vec![1]; // version
    bytes.extend(b"skip_this"); // data to skip
    bytes.push(b'\t'); // tab marker
    bytes.extend(b"after tab"); // actual content

    let (parsed, _) = TabDelimited::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.content, b"after tab");
}

#[test]
fn test_vec_of_vecs_with_newline_marker() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct MultiLine {
        line_count: u8,
        #[FromField(line_count)]
        #[UntilMarker('\n')]
        lines: Vec<Vec<u8>>,
    }

    let msg = MultiLine {
        line_count: 3,
        lines: vec![
            b"First line".to_vec(),
            b"Second line".to_vec(),
            b"Third line".to_vec(),
        ],
    };

    let bytes = msg.to_be_bytes();
    let (parsed, _) = MultiLine::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_carriage_return_marker() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct CRProtocol {
        msg_type: u8,
        #[UntilMarker('\r')]
        data: Vec<u8>,
        suffix: u16,
    }

    let msg = CRProtocol {
        msg_type: 5,
        data: b"carriage return test".to_vec(),
        suffix: 0xABCD,
    };

    let bytes = msg.to_be_bytes();
    let cr_pos = bytes.iter().position(|&b| b == b'\r').unwrap();
    assert_eq!(cr_pos, 1 + msg.data.len());

    let (parsed, _) = CRProtocol::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_mixed_byte_and_char_markers() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct MixedMarkers {
        start: u8,
        #[UntilMarker('\n')]
        first_section: Vec<u8>,
        #[UntilMarker(0xFF)]
        second_section: Vec<u8>,
        #[UntilMarker('\0')]
        third_section: Vec<u8>,
        end: u8,
    }

    let msg = MixedMarkers {
        start: 0x01,
        first_section: b"newline terminated".to_vec(),
        second_section: b"0xFF terminated".to_vec(),
        third_section: b"null terminated".to_vec(),
        end: 0x99,
    };

    let bytes = msg.to_be_bytes();
    let (parsed, _) = MixedMarkers::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, msg);
}
