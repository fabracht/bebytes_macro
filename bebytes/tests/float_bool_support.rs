use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
struct FloatStruct {
    f32_value: f32,
    f64_value: f64,
}

#[derive(BeBytes, Debug, PartialEq)]
struct BoolStruct {
    flag1: bool,
    flag2: bool,
    value: u16,
}

#[derive(BeBytes, Debug, PartialEq)]
struct MixedStruct {
    id: u32,
    temperature: f32,
    enabled: bool,
    ratio: f64,
    active: bool,
}

#[test]
fn test_f32_round_trip() {
    let values = [0.0f32, 1.0, -1.0, 3.14159, f32::MAX, f32::MIN, f32::EPSILON];

    for &val in &values {
        let s = FloatStruct {
            f32_value: val,
            f64_value: 0.0,
        };
        let bytes = s.to_be_bytes();
        let (parsed, _) = FloatStruct::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.f32_value, val, "f32 round-trip failed for {val}");
    }
}

#[test]
fn test_f64_round_trip() {
    let values = [
        0.0f64,
        1.0,
        -1.0,
        3.141592653589793,
        f64::MAX,
        f64::MIN,
        f64::EPSILON,
    ];

    for &val in &values {
        let s = FloatStruct {
            f32_value: 0.0,
            f64_value: val,
        };
        let bytes = s.to_be_bytes();
        let (parsed, _) = FloatStruct::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.f64_value, val, "f64 round-trip failed for {val}");
    }
}

#[test]
fn test_float_struct_size() {
    assert_eq!(FloatStruct::field_size(), 4 + 8);
}

#[test]
fn test_bool_round_trip() {
    let test_cases = [(false, false), (false, true), (true, false), (true, true)];

    for (flag1, flag2) in test_cases {
        let s = BoolStruct {
            flag1,
            flag2,
            value: 0x1234,
        };
        let bytes = s.to_be_bytes();
        let (parsed, _) = BoolStruct::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(parsed.flag1, flag1);
        assert_eq!(parsed.flag2, flag2);
        assert_eq!(parsed.value, 0x1234);
    }
}

#[test]
fn test_bool_strict_validation() {
    let valid_bytes = [0x00, 0x01, 0x12, 0x34];
    let (parsed, _) = BoolStruct::try_from_be_bytes(&valid_bytes).unwrap();
    assert!(!parsed.flag1);
    assert!(parsed.flag2);

    let invalid_bytes = [0x02, 0x00, 0x00, 0x00];
    let result = BoolStruct::try_from_be_bytes(&invalid_bytes);
    assert!(result.is_err(), "Should reject byte value 0x02 for bool");

    let invalid_bytes2 = [0x00, 0xFF, 0x00, 0x00];
    let result2 = BoolStruct::try_from_be_bytes(&invalid_bytes2);
    assert!(result2.is_err(), "Should reject byte value 0xFF for bool");
}

#[test]
fn test_bool_serialization_values() {
    let s_true = BoolStruct {
        flag1: true,
        flag2: false,
        value: 0,
    };
    let bytes = s_true.to_be_bytes();
    assert_eq!(bytes[0], 0x01, "true should serialize to 0x01");
    assert_eq!(bytes[1], 0x00, "false should serialize to 0x00");
}

#[test]
fn test_bool_struct_size() {
    assert_eq!(BoolStruct::field_size(), 1 + 1 + 2);
}

#[test]
fn test_mixed_struct_round_trip() {
    let s = MixedStruct {
        id: 12345,
        temperature: 98.6,
        enabled: true,
        ratio: 0.123456789,
        active: false,
    };

    let bytes = s.to_be_bytes();
    let (parsed, _) = MixedStruct::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(parsed.id, s.id);
    assert_eq!(parsed.temperature, s.temperature);
    assert_eq!(parsed.enabled, s.enabled);
    assert_eq!(parsed.ratio, s.ratio);
    assert_eq!(parsed.active, s.active);
}

#[test]
fn test_mixed_struct_size() {
    assert_eq!(MixedStruct::field_size(), 4 + 4 + 1 + 8 + 1);
}

#[test]
fn test_float_endianness() {
    let s = FloatStruct {
        f32_value: 1.0,
        f64_value: 2.0,
    };

    let be_bytes = s.to_be_bytes();
    let le_bytes = s.to_le_bytes();

    assert_ne!(be_bytes, le_bytes, "BE and LE should differ");

    let (be_parsed, _) = FloatStruct::try_from_be_bytes(&be_bytes).unwrap();
    let (le_parsed, _) = FloatStruct::try_from_le_bytes(&le_bytes).unwrap();

    assert_eq!(be_parsed.f32_value, s.f32_value);
    assert_eq!(le_parsed.f32_value, s.f32_value);
}

#[test]
fn test_float_nan_handling() {
    let s = FloatStruct {
        f32_value: f32::NAN,
        f64_value: f64::NAN,
    };

    let bytes = s.to_be_bytes();
    let (parsed, _) = FloatStruct::try_from_be_bytes(&bytes).unwrap();

    assert!(parsed.f32_value.is_nan());
    assert!(parsed.f64_value.is_nan());
}

#[test]
fn test_float_infinity() {
    let s = FloatStruct {
        f32_value: f32::INFINITY,
        f64_value: f64::NEG_INFINITY,
    };

    let bytes = s.to_be_bytes();
    let (parsed, _) = FloatStruct::try_from_be_bytes(&bytes).unwrap();

    assert!(parsed.f32_value.is_infinite() && parsed.f32_value.is_sign_positive());
    assert!(parsed.f64_value.is_infinite() && parsed.f64_value.is_sign_negative());
}
