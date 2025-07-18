use bebytes::BeBytes;
use test_case::test_case;

#[derive(BeBytes, Debug, PartialEq)]
pub struct ErrorEstimate {
    #[bits(1)]
    pub s_bit: u8,
    #[bits(1)]
    pub z_bit: u8,
    #[bits(6)]
    pub scale: u8,
    pub multiplier: u32,
}

#[test_case(0, 1, 0, 1; "s_bit_0_z_bit_1_scale_0_multiplier_1")]
#[test_case(1, 0, 63, 100; "s_bit_1_z_bit_0_scale_63_multiplier_100")]
fn test_new(s_bit: u8, z_bit: u8, scale: u8, multiplier: u32) {
    let error_estimate = ErrorEstimate::new(s_bit, z_bit, scale, multiplier);
    assert_eq!(
        error_estimate,
        ErrorEstimate {
            s_bit,
            z_bit,
            scale,
            multiplier,
        }
    );
}

#[test_case(&[0b01000000, 0b00000000, 0, 0, 1], ErrorEstimate { s_bit: 0, z_bit: 1, scale: 0, multiplier: 1 }; "input1")]
#[test_case(&[0b10111111, 0b00000000, 0, 0, 100], ErrorEstimate { s_bit: 1, z_bit: 0, scale: 63, multiplier: 100 }; "input2")]
fn test_try_from_be_bytes(input: &[u8], expected: ErrorEstimate) {
    let error_estimate = ErrorEstimate::try_from_be_bytes(input).unwrap();
    assert_eq!(error_estimate.0, expected);
}

#[test_case(ErrorEstimate { s_bit: 0, z_bit: 1, scale: 0, multiplier: 1 }, vec![0b01000000, 0b00000000, 0, 0, 1]; "input1")]
#[test_case(ErrorEstimate { s_bit: 1, z_bit: 0, scale: 63, multiplier: 100 }, vec![0b10111111, 0b00000000, 0, 0, 100]; "input2")]
fn test_to_be_bytes(input: ErrorEstimate, expected: Vec<u8>) {
    let bytes = input.to_be_bytes();
    assert_eq!(bytes, expected);
}

#[test_case(ErrorEstimate { s_bit: 0, z_bit: 1, scale: 0, multiplier: 1 }, vec![2, 1, 0, 0, 0]; "le_input1")]
#[test_case(ErrorEstimate { s_bit: 1, z_bit: 0, scale: 63, multiplier: 100 }, vec![253, 100, 0, 0, 0]; "le_input2")]
fn test_to_le_bytes(input: ErrorEstimate, expected: Vec<u8>) {
    let bytes = input.to_le_bytes();
    assert_eq!(bytes, expected);
}

#[test_case(&[2, 1, 0, 0, 0], ErrorEstimate { s_bit: 0, z_bit: 1, scale: 0, multiplier: 1 }; "le_input1")]
#[test_case(&[253, 100, 0, 0, 0], ErrorEstimate { s_bit: 1, z_bit: 0, scale: 63, multiplier: 100 }; "le_input2")]
fn test_try_from_le_bytes(input: &[u8], expected: ErrorEstimate) {
    let error_estimate = ErrorEstimate::try_from_le_bytes(input).unwrap();
    assert_eq!(error_estimate.0, expected);
}

#[test]
fn test_size() {
    assert_eq!(ErrorEstimate::field_size(), 5);
}

#[test]
fn test_endianness_round_trip() {
    let error_estimate = ErrorEstimate {
        s_bit: 1,
        z_bit: 0,
        scale: 63,
        multiplier: 16777215,
    };

    // Big endian round trip
    let be_bytes = error_estimate.to_be_bytes();
    let (be_result, _) = ErrorEstimate::try_from_be_bytes(&be_bytes).unwrap();
    assert_eq!(error_estimate, be_result);

    // Little endian round trip
    let le_bytes = error_estimate.to_le_bytes();
    let (le_result, _) = ErrorEstimate::try_from_le_bytes(&le_bytes).unwrap();
    assert_eq!(error_estimate, le_result);
}
