use bebytes::BeBytes;
use test_case::test_case;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use core::fmt::Write;

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

#[test]
fn test_endian_conversion() {
    let original = ErrorEstimate {
        s_bit: 1,
        z_bit: 0,
        scale: 63,
        multiplier: 100,
    };

    // Convert to big-endian
    let be_bytes = original.to_be_bytes();
    // Read back from big-endian
    let (from_be, _) = ErrorEstimate::try_from_be_bytes(&be_bytes).unwrap();
    assert_eq!(original, from_be);

    // Convert to little-endian
    let le_bytes = original.to_le_bytes();
    // Read back from little-endian
    let (from_le, _) = ErrorEstimate::try_from_le_bytes(&le_bytes).unwrap();
    assert_eq!(original, from_le);

    // Ensure the byte representations are different
    assert_ne!(be_bytes, le_bytes);

    // But trying to read big-endian data as little-endian should give incorrect results
    let (wrong_endian, _) = ErrorEstimate::try_from_le_bytes(&be_bytes).unwrap();
    assert_ne!(original, wrong_endian);
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
    #[bits(1)]
    pub s_bit: u8,
    #[bits(1)]
    pub z_bit: u8,
    #[bits(6)]
    pub scale: u8,
    pub dummy_struct: DummyStruct,
}

#[derive(BeBytes, Debug, PartialEq, Clone, Default)]
pub struct DummyStruct {
    pub dummy0: [u8; 2],
    #[bits(1)]
    pub dummy1: u8,
    #[bits(7)]
    pub dummy2: u8,
}

#[test_case(NestedStruct::default(), 4; "test nested struct")]
fn test_nested_struct(input: NestedStruct, len: usize) {
    let bytes = input.clone().to_be_bytes();
    for byte in bytes.iter() {
        println!("{:08b} ", *byte);
    }
    println!("bytes: {:?}, len: {}", bytes, bytes.len());
    let (nested_struct, written_len) = NestedStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(nested_struct, input);
    assert_eq!(len, written_len);
}

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct VecLengthFromFieldName {
    pre_tail: u8,
    #[FromField(pre_tail)]
    tail: Vec<u8>,
    post_tail: u8,
}

#[test_case(VecLengthFromFieldName { pre_tail: 2, tail: vec![2, 3], post_tail: 4 }, 4; "test vec length from field name")]
fn test_vec_length_from_field_name(input: VecLengthFromFieldName, len: usize) {
    let bytes = input.clone().to_be_bytes();
    let (vec_length_from_field_name, written_len) =
        VecLengthFromFieldName::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(vec_length_from_field_name, input);
    assert_eq!(len, written_len);
    // Check that we read the number of bytes specified by the pre_tail field
    let bytes = vec![2, 2, 3, 4];
    let (vec_length_from_field_name, _written_len) =
        VecLengthFromFieldName::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(vec_length_from_field_name.tail.len(), 2);
}

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct PaddingVecWithVecLengthFromFieldName {
    innocent: u8,
    mid_tail: VecLengthFromFieldName,
    real_tail: Vec<u8>,
}

#[test_case(PaddingVecWithVecLengthFromFieldName { innocent: 1, mid_tail: VecLengthFromFieldName { pre_tail: 2, tail: vec![2, 3], post_tail: 4 }, real_tail: vec![5, 6] }, 7; "test padding vec with vec length from field name")]
fn test_padding_vec_with_vec_length_from_field_name(
    input: PaddingVecWithVecLengthFromFieldName,
    len: usize,
) {
    let bytes = input.clone().to_be_bytes();
    let (padding_vec_with_vec_length_from_field_name, written_len) =
        PaddingVecWithVecLengthFromFieldName::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(padding_vec_with_vec_length_from_field_name, input.clone());
    assert_eq!(len, written_len);
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct WithSizeAttribute {
    #[With(size(3))]
    with_size: Vec<u8>,
    fourth: u8,
}

#[test_case(WithSizeAttribute { with_size: vec![1, 2, 3], fourth: 4 }, 4; "test with size attribute")]
fn test_with_size_attribute(input: WithSizeAttribute, len: usize) {
    let bytes = input.clone().to_be_bytes();
    let (with_size_attribute, written_len) = WithSizeAttribute::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(with_size_attribute, input);
    assert_eq!(len, written_len);
    // Test that we read the number of bytes specified by the size attribute
    let bytes = vec![1, 2, 3, 4, 5, 6];
    let (with_size_attribute, _written_len) = WithSizeAttribute::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(
        with_size_attribute,
        WithSizeAttribute {
            with_size: vec![1, 2, 3],
            fourth: 4
        }
    );
}

#[derive(BeBytes, Debug, PartialEq)]
struct U16 {
    #[bits(1)]
    first: u8,
    #[bits(14)]
    second: u16,
    #[bits(1)]
    fourth: u8,
}

#[test_case(U16 { first: 1, second: 2, fourth: 1 }, 2; "test u16")]
fn test_u16(input: U16, len: usize) {
    let bytes = input.to_be_bytes();
    let (u16, written_len) = U16::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(u16, input);
    assert_eq!(len, written_len);
}

#[derive(BeBytes, Debug, PartialEq)]
struct U32 {
    #[bits(1)]
    first: u8,
    #[bits(30)]
    second: u32,
    #[bits(1)]
    fourth: u8,
}

#[test_case(U32 { first: 1, second: 2, fourth: 1 }, 4; "test u32")]
fn test_u32(input: U32, len: usize) {
    let bytes = input.to_be_bytes();
    let (u32, written_len) = U32::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(u32, input);
    assert_eq!(len, written_len);
}

#[derive(BeBytes, Debug, PartialEq)]
struct U64 {
    #[bits(1)]
    first: u8,
    #[bits(62)]
    second: u64,
    #[bits(1)]
    fourth: u8,
}

#[test_case(U64 { first: 1, second: 3, fourth: 1 }, 8; "test u64")]
fn test_u64(input: U64, len: usize) {
    let bytes = input.to_be_bytes();
    let (u64, written_len) = U64::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(u64, input);
    assert_eq!(len, written_len);
}

#[derive(BeBytes, Debug, PartialEq)]
struct U128 {
    #[bits(1)]
    first: u8,
    #[bits(126)]
    second: u128,
    #[bits(1)]
    fourth: u8,
}

#[test_case(U128 { first: 1, second: 33, fourth: 1 }, 16; "test u128")]
fn test_u128(input: U128, len: usize) {
    let bytes = input.to_be_bytes();
    let (u128, written_len) = U128::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(u128, input);
    assert_eq!(len, written_len);
}

#[derive(BeBytes, Debug, PartialEq)]
struct I8 {
    #[bits(1)]
    first: u8,
    #[bits(6)]
    second: i8,
    #[bits(1)]
    fourth: u8,
}

#[test_case(I8 { first: 1, second: 3, fourth: 1 }, 1; "test i8")]
fn test_i8(input: I8, len: usize) {
    let bytes = input.to_be_bytes();
    let (i8, written_len) = I8::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(i8, input);
    assert_eq!(len, written_len);
}

#[derive(BeBytes, Debug, PartialEq)]
struct I16 {
    #[bits(1)]
    first: u8,
    #[bits(14)]
    second: i16,
    #[bits(1)]
    fourth: u8,
}

#[test_case(I16 { first: 1, second: 3, fourth: 1 }, 2; "test i16")]
fn test_i16(input: I16, len: usize) {
    let bytes = input.to_be_bytes();
    let (i16, written_len) = I16::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(i16, input);
    assert_eq!(len, written_len);
}

#[derive(BeBytes, Debug, PartialEq)]
struct I32 {
    #[bits(1)]
    first: u8,
    #[bits(30)]
    second: i32,
    #[bits(1)]
    fourth: u8,
}

#[test_case(I32 { first: 1, second: 3, fourth: 1 }, 4; "test i32")]
fn test_i32(input: I32, len: usize) {
    let bytes = input.to_be_bytes();
    let (i32, written_len) = I32::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(i32, input);
    assert_eq!(len, written_len);
}

#[derive(BeBytes, Debug, PartialEq)]
struct I64 {
    #[bits(1)]
    first: u8,
    #[bits(62)]
    second: i64,
    #[bits(1)]
    fourth: u8,
}

#[test_case(I64 { first: 1, second: 3, fourth: 1 }, 8; "test i64")]
fn test_i64(input: I64, len: usize) {
    let bytes = input.to_be_bytes();
    let (i64, written_len) = I64::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(i64, input);
    assert_eq!(len, written_len);
}

#[derive(BeBytes, Debug, PartialEq)]
struct I128 {
    #[bits(1)]
    first: u8,
    #[bits(126)]
    second: i128,
    #[bits(1)]
    fourth: u8,
}

#[test_case(I128 { first: 1, second: 3, fourth: 1 }, 16; "test i128")]
fn test_i128(input: I128, len: usize) {
    let bytes = input.to_be_bytes();
    let (i128, written_len) = I128::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(i128, input);
    assert_eq!(len, written_len);
}
