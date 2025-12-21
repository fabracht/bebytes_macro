use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
struct OptionU8 {
    value: Option<u8>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionU16 {
    value: Option<u16>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionU32 {
    value: Option<u32>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionU64 {
    value: Option<u64>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionU128 {
    value: Option<u128>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionI8 {
    value: Option<i8>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionI16 {
    value: Option<i16>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionI32 {
    value: Option<i32>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionI64 {
    value: Option<i64>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionI128 {
    value: Option<i128>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionF32 {
    value: Option<f32>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionF64 {
    value: Option<f64>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionBool {
    value: Option<bool>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionChar {
    value: Option<char>,
}

#[test]
fn test_option_u8_some_zero_disambiguated() {
    let some_zero = OptionU8 { value: Some(0) };
    let none = OptionU8 { value: None };

    let some_zero_bytes = some_zero.to_be_bytes();
    let none_bytes = none.to_be_bytes();

    assert_ne!(some_zero_bytes, none_bytes);
    assert_eq!(some_zero_bytes, [0x01, 0x00]);
    assert_eq!(none_bytes, [0x00, 0x00]);

    let (parsed_some, _) = OptionU8::try_from_be_bytes(&some_zero_bytes).unwrap();
    let (parsed_none, _) = OptionU8::try_from_be_bytes(&none_bytes).unwrap();

    assert_eq!(parsed_some.value, Some(0));
    assert_eq!(parsed_none.value, None);
}

#[test]
fn test_option_u8_some_value() {
    let s = OptionU8 { value: Some(42) };
    let bytes = s.to_be_bytes();
    assert_eq!(bytes, [0x01, 42]);

    let (parsed, consumed) = OptionU8::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.value, Some(42));
    assert_eq!(consumed, 2);
}

#[test]
fn test_option_u16_round_trip() {
    for val in [0u16, 1, 255, 256, u16::MAX] {
        let s = OptionU16 { value: Some(val) };
        let bytes = s.to_be_bytes();
        assert_eq!(bytes[0], 0x01);

        let (parsed, consumed) = OptionU16::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value, Some(val));
        assert_eq!(consumed, 3);
    }
}

#[test]
fn test_option_u32_round_trip() {
    for val in [0u32, 1, u32::MAX] {
        let s = OptionU32 { value: Some(val) };
        let bytes = s.to_be_bytes();
        let (parsed, _) = OptionU32::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value, Some(val));
    }
}

#[test]
fn test_option_u64_round_trip() {
    for val in [0u64, 1, u64::MAX] {
        let s = OptionU64 { value: Some(val) };
        let bytes = s.to_be_bytes();
        let (parsed, _) = OptionU64::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value, Some(val));
    }
}

#[test]
fn test_option_u128_round_trip() {
    for val in [0u128, 1, u128::MAX] {
        let s = OptionU128 { value: Some(val) };
        let bytes = s.to_be_bytes();
        let (parsed, _) = OptionU128::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value, Some(val));
    }
}

#[test]
fn test_option_i8_round_trip() {
    for val in [i8::MIN, -1, 0, 1, i8::MAX] {
        let s = OptionI8 { value: Some(val) };
        let bytes = s.to_be_bytes();
        let (parsed, _) = OptionI8::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value, Some(val));
    }
}

#[test]
fn test_option_i16_round_trip() {
    for val in [i16::MIN, -1, 0, 1, i16::MAX] {
        let s = OptionI16 { value: Some(val) };
        let bytes = s.to_be_bytes();
        let (parsed, _) = OptionI16::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value, Some(val));
    }
}

#[test]
fn test_option_i32_round_trip() {
    for val in [i32::MIN, -1, 0, 1, i32::MAX] {
        let s = OptionI32 { value: Some(val) };
        let bytes = s.to_be_bytes();
        let (parsed, _) = OptionI32::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value, Some(val));
    }
}

#[test]
fn test_option_i64_round_trip() {
    for val in [i64::MIN, -1, 0, 1, i64::MAX] {
        let s = OptionI64 { value: Some(val) };
        let bytes = s.to_be_bytes();
        let (parsed, _) = OptionI64::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value, Some(val));
    }
}

#[test]
fn test_option_i128_round_trip() {
    for val in [i128::MIN, -1, 0, 1, i128::MAX] {
        let s = OptionI128 { value: Some(val) };
        let bytes = s.to_be_bytes();
        let (parsed, _) = OptionI128::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value, Some(val));
    }
}

#[test]
fn test_option_f32_round_trip() {
    for val in [0.0f32, 1.0, -1.0, 3.14159, f32::MAX, f32::MIN] {
        let s = OptionF32 { value: Some(val) };
        let bytes = s.to_be_bytes();
        assert_eq!(bytes[0], 0x01);
        let (parsed, _) = OptionF32::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value, Some(val));
    }
}

#[test]
fn test_option_f32_none() {
    let s = OptionF32 { value: None };
    let bytes = s.to_be_bytes();
    assert_eq!(bytes[0], 0x00);
    let (parsed, _) = OptionF32::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.value, None);
}

#[test]
fn test_option_f64_round_trip() {
    for val in [0.0f64, 1.0, -1.0, f64::MAX, f64::MIN] {
        let s = OptionF64 { value: Some(val) };
        let bytes = s.to_be_bytes();
        let (parsed, _) = OptionF64::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value, Some(val));
    }
}

#[test]
fn test_option_f64_none() {
    let s = OptionF64 { value: None };
    let bytes = s.to_be_bytes();
    assert_eq!(bytes[0], 0x00);
    let (parsed, _) = OptionF64::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.value, None);
}

#[test]
fn test_option_bool_some_true() {
    let s = OptionBool { value: Some(true) };
    let bytes = s.to_be_bytes();
    assert_eq!(bytes, [0x01, 0x01]);
    let (parsed, _) = OptionBool::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.value, Some(true));
}

#[test]
fn test_option_bool_some_false() {
    let s = OptionBool { value: Some(false) };
    let bytes = s.to_be_bytes();
    assert_eq!(bytes, [0x01, 0x00]);
    let (parsed, _) = OptionBool::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.value, Some(false));
}

#[test]
fn test_option_bool_none() {
    let s = OptionBool { value: None };
    let bytes = s.to_be_bytes();
    assert_eq!(bytes, [0x00, 0x00]);
    let (parsed, _) = OptionBool::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.value, None);
}

#[test]
fn test_option_char_round_trip() {
    for val in ['A', 'z', '\u{1F600}', '\0'] {
        let s = OptionChar { value: Some(val) };
        let bytes = s.to_be_bytes();
        assert_eq!(bytes[0], 0x01);
        let (parsed, _) = OptionChar::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value, Some(val));
    }
}

#[test]
fn test_option_char_none() {
    let s = OptionChar { value: None };
    let bytes = s.to_be_bytes();
    assert_eq!(bytes[0], 0x00);
    let (parsed, _) = OptionChar::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.value, None);
}

#[test]
fn test_option_field_sizes() {
    assert_eq!(OptionU8::field_size(), 2);
    assert_eq!(OptionU16::field_size(), 3);
    assert_eq!(OptionU32::field_size(), 5);
    assert_eq!(OptionU64::field_size(), 9);
    assert_eq!(OptionU128::field_size(), 17);
    assert_eq!(OptionI8::field_size(), 2);
    assert_eq!(OptionI16::field_size(), 3);
    assert_eq!(OptionI32::field_size(), 5);
    assert_eq!(OptionI64::field_size(), 9);
    assert_eq!(OptionI128::field_size(), 17);
    assert_eq!(OptionF32::field_size(), 5);
    assert_eq!(OptionF64::field_size(), 9);
    assert_eq!(OptionBool::field_size(), 2);
    assert_eq!(OptionChar::field_size(), 5);
}

#[test]
fn test_invalid_option_tag() {
    let invalid_bytes = [0x02, 0x00];
    let result = OptionU8::try_from_be_bytes(&invalid_bytes);
    assert!(result.is_err());
}

#[test]
fn test_option_char_invalid_unicode() {
    let invalid_bytes = [0x01, 0x00, 0xD8, 0x00, 0x00];
    let result = OptionChar::try_from_be_bytes(&invalid_bytes);
    assert!(result.is_err());
}

#[test]
fn test_option_in_mixed_struct() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct MixedStruct {
        prefix: u8,
        opt_value: Option<u16>,
        suffix: u32,
    }

    let s = MixedStruct {
        prefix: 0xAA,
        opt_value: Some(0x1234),
        suffix: 0xDEADBEEF,
    };

    let bytes = s.to_be_bytes();
    assert_eq!(bytes.len(), 1 + 3 + 4);
    assert_eq!(bytes[0], 0xAA);
    assert_eq!(bytes[1], 0x01);
    assert_eq!(bytes[2..4], [0x12, 0x34]);
    assert_eq!(bytes[4..8], [0xDE, 0xAD, 0xBE, 0xEF]);

    let (parsed, _) = MixedStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, s);
}

#[test]
fn test_option_none_in_mixed_struct() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct MixedStruct {
        prefix: u8,
        opt_value: Option<u16>,
        suffix: u32,
    }

    let s = MixedStruct {
        prefix: 0xAA,
        opt_value: None,
        suffix: 0xDEADBEEF,
    };

    let bytes = s.to_be_bytes();
    assert_eq!(bytes.len(), 1 + 3 + 4);
    assert_eq!(bytes[0], 0xAA);
    assert_eq!(bytes[1], 0x00);
    assert_eq!(bytes[2..4], [0x00, 0x00]);
    assert_eq!(bytes[4..8], [0xDE, 0xAD, 0xBE, 0xEF]);

    let (parsed, _) = MixedStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, s);
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionArray4 {
    value: Option<[u8; 4]>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct OptionArray16 {
    value: Option<[u8; 16]>,
}

#[test]
fn test_option_array_some() {
    let s = OptionArray4 {
        value: Some([1, 2, 3, 4]),
    };
    let bytes = s.to_be_bytes();
    assert_eq!(bytes.len(), 5);
    assert_eq!(bytes[0], 0x01);
    assert_eq!(&bytes[1..5], &[1, 2, 3, 4]);

    let (parsed, consumed) = OptionArray4::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.value, Some([1, 2, 3, 4]));
    assert_eq!(consumed, 5);
}

#[test]
fn test_option_array_none() {
    let s = OptionArray4 { value: None };
    let bytes = s.to_be_bytes();
    assert_eq!(bytes.len(), 5);
    assert_eq!(bytes[0], 0x00);
    assert_eq!(&bytes[1..5], &[0, 0, 0, 0]);

    let (parsed, consumed) = OptionArray4::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.value, None);
    assert_eq!(consumed, 5);
}

#[test]
fn test_option_array_field_size() {
    assert_eq!(OptionArray4::field_size(), 5);
    assert_eq!(OptionArray16::field_size(), 17);
}

#[test]
fn test_option_array_round_trip() {
    let values: [Option<[u8; 4]>; 3] = [Some([0, 0, 0, 0]), Some([255, 255, 255, 255]), None];

    for value in values {
        let s = OptionArray4 { value };
        let bytes = s.to_be_bytes();
        let (parsed, _) = OptionArray4::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.value, value);
    }
}

#[test]
fn test_option_array_in_struct() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct MixedWithArray {
        prefix: u8,
        opt_array: Option<[u8; 4]>,
        suffix: u16,
    }

    let s = MixedWithArray {
        prefix: 0xAA,
        opt_array: Some([1, 2, 3, 4]),
        suffix: 0xBBCC,
    };

    let bytes = s.to_be_bytes();
    assert_eq!(bytes.len(), 1 + 5 + 2);
    assert_eq!(bytes[0], 0xAA);
    assert_eq!(bytes[1], 0x01);
    assert_eq!(&bytes[2..6], &[1, 2, 3, 4]);
    assert_eq!(&bytes[6..8], &[0xBB, 0xCC]);

    let (parsed, _) = MixedWithArray::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed, s);
}

#[test]
fn test_option_array_invalid_tag() {
    let invalid_bytes = [0x02, 0x00, 0x00, 0x00, 0x00];
    let result = OptionArray4::try_from_be_bytes(&invalid_bytes);
    assert!(result.is_err());
}

#[test]
fn test_option_little_endian_round_trip() {
    let s = OptionU32 {
        value: Some(0x12345678),
    };
    let bytes = s.to_le_bytes();
    assert_eq!(bytes.len(), 5);
    assert_eq!(bytes[0], 0x01);
    assert_eq!(&bytes[1..5], &[0x78, 0x56, 0x34, 0x12]);

    let (parsed, _) = OptionU32::try_from_le_bytes(&bytes).unwrap();
    assert_eq!(parsed.value, Some(0x12345678));

    let none = OptionU32 { value: None };
    let none_bytes = none.to_le_bytes();
    assert_eq!(none_bytes[0], 0x00);

    let (parsed_none, _) = OptionU32::try_from_le_bytes(&none_bytes).unwrap();
    assert_eq!(parsed_none.value, None);
}
