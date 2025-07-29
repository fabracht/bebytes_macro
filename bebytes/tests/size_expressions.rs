use bebytes::BeBytes as _;
use bebytes_derive::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
struct MathExpressions {
    count: u8,
    #[With(size(count * 4))]
    multiply_data: Vec<u8>,
    base: u8,
    #[With(size(base + 2))]
    add_data: Vec<u8>,
    divisor: u8,
    #[With(size(12 / divisor))]
    divide_data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct FieldReferences {
    length1: u8,
    #[With(size(length1))]
    data1: String,
    length2: u16,
    #[With(size(length2))]
    data2: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct NestedExpressions {
    a: u8,
    b: u8,
    #[With(size(a * b + 1))]
    result_data: Vec<u8>,
}

#[test]
fn test_mathematical_expressions() {
    let msg = MathExpressions {
        count: 3,
        multiply_data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], // 3 * 4 = 12 bytes
        base: 5,
        add_data: vec![20, 21, 22, 23, 24, 25, 26], // 5 + 2 = 7 bytes
        divisor: 4,
        divide_data: vec![30, 31, 32], // 12 / 4 = 3 bytes
    };

    let bytes = msg.to_be_bytes();
    let (parsed, _) = MathExpressions::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn test_field_references() {
    let msg = FieldReferences {
        length1: 6,
        data1: "Hello!".to_string(),
        length2: 4,
        data2: vec![100, 101, 102, 103],
    };

    let bytes = msg.to_be_bytes();
    let (parsed, _) = FieldReferences::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn test_nested_expressions() {
    let msg = NestedExpressions {
        a: 3,
        b: 2,
        result_data: vec![1, 2, 3, 4, 5, 6, 7], // 3 * 2 + 1 = 7 bytes
    };

    let bytes = msg.to_be_bytes();
    let (parsed, _) = NestedExpressions::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn test_zero_size_expression() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct ZeroSize {
        count: u8,
        #[With(size(count * 0))]
        empty_data: Vec<u8>,
    }

    let msg = ZeroSize {
        count: 5,
        empty_data: vec![],
    };

    let bytes = msg.to_be_bytes();
    let (parsed, _) = ZeroSize::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn test_string_expressions() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct StringExprs {
        len: u8,
        #[With(size(len))]
        message: String,
        padding: u8,
        #[With(size(len + padding))]
        padded_message: String,
    }

    let msg = StringExprs {
        len: 5,
        message: "Hello".to_string(),
        padding: 3,
        padded_message: "Hello   ".to_string(), // 5 + 3 = 8 chars
    };

    let bytes = msg.to_be_bytes();
    let (parsed, _) = StringExprs::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn test_serialization_deserialization_consistency() {
    let original = MathExpressions {
        count: 2,
        multiply_data: vec![10, 20, 30, 40, 50, 60, 70, 80], // 2 * 4 = 8 bytes
        base: 3,
        add_data: vec![1, 2, 3, 4, 5], // 3 + 2 = 5 bytes
        divisor: 3,
        divide_data: vec![100, 101, 102, 103], // 12 / 3 = 4 bytes
    };

    // Test big-endian
    let be_bytes = original.to_be_bytes();
    let (be_parsed, be_size) = MathExpressions::try_from_be_bytes(&be_bytes).unwrap();
    assert_eq!(original, be_parsed);
    assert_eq!(be_size, be_bytes.len());

    // Test little-endian
    let le_bytes = original.to_le_bytes();
    let (le_parsed, le_size) = MathExpressions::try_from_le_bytes(&le_bytes).unwrap();
    assert_eq!(original, le_parsed);
    assert_eq!(le_size, le_bytes.len());
}

#[test]
#[should_panic(expected = "Vector size")]
fn test_size_mismatch_panic() {
    let msg = MathExpressions {
        count: 3,
        multiply_data: vec![1, 2, 3, 4, 5], // Wrong size: should be 3 * 4 = 12 bytes, but is 5
        base: 5,
        add_data: vec![20, 21, 22, 23, 24, 25, 26],
        divisor: 4,
        divide_data: vec![30, 31, 32],
    };

    // This should panic because multiply_data has wrong size
    let _bytes = msg.to_be_bytes();
}

#[test]
fn test_insufficient_data_error() {
    let insufficient_bytes = vec![1, 2]; // Not enough data
    let result = MathExpressions::try_from_be_bytes(&insufficient_bytes);
    assert!(result.is_err());

    if let Err(bebytes::BeBytesError::InsufficientData { expected, actual }) = result {
        assert!(expected > actual);
    } else {
        panic!("Expected InsufficientData error");
    }
}
