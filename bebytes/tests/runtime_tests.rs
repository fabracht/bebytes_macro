use bebytes::BeBytes;
use test_case::test_case;

#[derive(BeBytes, Debug, PartialEq)]
pub struct ErrorEstimate {
    #[U8(size(1), pos(0))]
    pub s_bit: u8,
    #[U8(size(1), pos(1))]
    pub z_bit: u8,
    #[U8(size(6), pos(2))]
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

#[test]
#[should_panic(expected = "Value of field scale is out of range")]
fn test_value_out_of_range() {
    let _ = ErrorEstimate::new(0, 1, 64, 1);
}

#[derive(BeBytes, Copy, Clone, Eq, PartialEq, Debug)]
pub struct ClientSetupResponse {
    pub mode: Modes,
    pub key_id: [u8; 1],
    pub token: [u8; 1],
    pub client_iv: [u8; 1],
}

#[derive(BeBytes, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Modes {
    pub bits: u8,
}

#[test_case(ClientSetupResponse { mode: Modes { bits: 1 }, key_id: [0; 1], token: [0; 1], client_iv: [0; 1] }, 4; "test arrays length")]
fn test_arrays(input: ClientSetupResponse, len: usize) {
    let bytes = input.clone().to_be_bytes();
    let (client_setup_response, written_len) =
        ClientSetupResponse::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(client_setup_response, input);
    assert_eq!(len, written_len);
}

#[derive(BeBytes, Debug, PartialEq, Clone, Default)]
pub struct NestedStruct {
    #[U8(size(1), pos(0))]
    pub s_bit: u8,
    #[U8(size(1), pos(1))]
    pub z_bit: u8,
    #[U8(size(6), pos(2))]
    pub scale: u8,
    pub dummy_struct: DummyStruct,
}

#[derive(BeBytes, Debug, PartialEq, Clone, Default)]
pub struct DummyStruct {
    pub dummy0: [u8; 2],
    #[U8(size(1), pos(0))]
    pub dummy1: u8,
    #[U8(size(7), pos(1))]
    pub dummy2: u8,
}

#[test_case(NestedStruct::default(), 4; "test nested struct")]
fn test_nested_struct(input: NestedStruct, len: usize) {
    let bytes = input.clone().to_be_bytes();
    for byte in bytes.iter() {
        print!("{:08b} ", *byte);
        print!("\n");
    }
    println!("bytes: {:?}, len: {}", bytes, bytes.len());
    let (nested_struct, written_len) = NestedStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(nested_struct, input);
    assert_eq!(len, written_len);
}
