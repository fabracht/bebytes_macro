use bebytes::BeBytes;
use proptest::prelude::*;

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct Add {
    base: u8,
    #[With(size(base + 10))]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct Mul {
    a: u8,
    b: u8,
    #[With(size(a * b))]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct Div {
    base: u8,
    #[With(size(100 / base))]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct Rem {
    base: u8,
    #[With(size(base % 4))]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct RemByField {
    base: u8,
    modulus: u8,
    #[With(size(base % modulus))]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct Sub {
    count: u8,
    #[With(size(count - 10))]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct Paren {
    w: u8,
    h: u8,
    bpp: u8,
    #[With(size((w * h) * bpp))]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct Precedence {
    a: u8,
    b: u8,
    c: u8,
    #[With(size(a + b * c))]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct WideMul {
    a: u64,
    b: u64,
    #[With(size(a * b))]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct WideAdd {
    a: u64,
    b: u64,
    #[With(size(a + b))]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct SignedLength {
    len: i32,
    #[With(size(len))]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct ExprString {
    len: u64,
    #[With(size(len))]
    data: String,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct ExprVec {
    len: u64,
    #[With(size(len))]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct FromFieldVec {
    len: u64,
    #[FromField(len)]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct FromFieldString {
    len: u64,
    #[FromField(len)]
    data: String,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct FromFieldSignedVec {
    len: i64,
    #[FromField(len)]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct FromFieldSections {
    count: u64,
    #[FromField(count)]
    #[UntilMarker(0xFF)]
    sections: Vec<Vec<u8>>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct Item {
    id: u8,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct FromFieldItems {
    count: u64,
    #[FromField(count)]
    items: Vec<Item>,
}

fn consumed<T: BeBytes>(bytes: &[u8]) -> usize {
    T::try_from_be_bytes(bytes)
        .unwrap_or_else(|error| panic!("expected Ok, got {error}"))
        .1
}

fn assert_rejected<T: BeBytes>(bytes: &[u8]) {
    assert!(
        T::try_from_be_bytes(bytes).is_err(),
        "expected Err for {bytes:?}"
    );
    assert!(
        T::try_from_le_bytes(bytes).is_err(),
        "expected Err for {bytes:?} in little endian"
    );
}

fn with_tail(head: &[u8], tail: &[u8]) -> Vec<u8> {
    let mut bytes = head.to_vec();
    bytes.extend_from_slice(tail);
    bytes
}

#[test]
fn add_parses() {
    assert_eq!(
        consumed::<Add>(&[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        13
    );
}

#[test]
fn mul_parses() {
    assert_eq!(consumed::<Mul>(&[2, 3, 1, 2, 3, 4, 5, 6, 7]), 8);
}

#[test]
fn div_parses() {
    assert_eq!(consumed::<Div>(&[50, 9, 9]), 3);
}

#[test]
fn rem_parses() {
    assert_eq!(consumed::<Rem>(&[7, 1, 2, 3]), 4);
}

#[test]
fn sub_parses() {
    assert_eq!(consumed::<Sub>(&[12, 1, 2]), 3);
}

#[test]
fn paren_parses() {
    assert_eq!(
        consumed::<Paren>(&[2, 3, 2, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
        15
    );
}

#[test]
fn precedence_parses() {
    assert_eq!(consumed::<Precedence>(&[1, 2, 3, 9, 9, 9, 9, 9, 9, 9]), 10);
}

#[test]
fn division_by_zero_field_is_rejected() {
    assert_rejected::<Div>(&[0, 1, 2, 3]);
}

#[test]
fn remainder_by_zero_field_is_rejected() {
    assert_rejected::<RemByField>(&[7, 0, 1, 2, 3]);
}

#[test]
fn subtraction_underflow_is_rejected() {
    assert_rejected::<Sub>(&[3, 1, 2, 3]);
}

#[test]
fn multiplication_overflow_is_rejected() {
    let bytes = with_tail(
        &[u64::MAX.to_be_bytes(), 2u64.to_be_bytes()].concat(),
        &[1, 2, 3],
    );
    assert_rejected::<WideMul>(&bytes);
}

#[test]
fn addition_overflow_is_rejected() {
    let bytes = with_tail(
        &[u64::MAX.to_be_bytes(), 1u64.to_be_bytes()].concat(),
        &[1, 2, 3],
    );
    assert_rejected::<WideAdd>(&bytes);
}

#[test]
fn negative_length_is_rejected() {
    let bytes = with_tail(&(-1i32).to_be_bytes(), &[1, 2, 3]);
    assert_rejected::<SignedLength>(&bytes);
}

#[test]
fn huge_expression_vec_length_is_rejected() {
    let bytes = with_tail(&u64::MAX.to_be_bytes(), &[1, 2, 3]);
    assert_rejected::<ExprVec>(&bytes);
}

#[test]
fn huge_expression_string_length_is_rejected() {
    let bytes = with_tail(&u64::MAX.to_be_bytes(), &[1, 2, 3]);
    assert_rejected::<ExprString>(&bytes);
}

#[test]
fn huge_from_field_vec_length_is_rejected() {
    let bytes = with_tail(&u64::MAX.to_be_bytes(), &[1, 2, 3]);
    assert_rejected::<FromFieldVec>(&bytes);
}

#[test]
fn huge_from_field_string_length_is_rejected() {
    let bytes = with_tail(&u64::MAX.to_be_bytes(), &[1, 2, 3]);
    assert_rejected::<FromFieldString>(&bytes);
}

#[test]
fn negative_from_field_length_is_rejected() {
    let bytes = with_tail(&(-1i64).to_be_bytes(), &[1, 2, 3]);
    assert_rejected::<FromFieldSignedVec>(&bytes);
}

#[test]
fn huge_from_field_section_count_is_rejected() {
    let bytes = with_tail(&u64::MAX.to_be_bytes(), &[1, 0xFF, 2, 0xFF]);
    assert_rejected::<FromFieldSections>(&bytes);
}

#[test]
fn huge_from_field_item_count_is_rejected() {
    let bytes = with_tail(&u64::MAX.to_be_bytes(), &[1, 2, 3]);
    assert_rejected::<FromFieldItems>(&bytes);
}

#[test]
fn huge_length_reports_insufficient_data() {
    let bytes = with_tail(&u64::MAX.to_be_bytes(), &[1, 2, 3]);
    for error in [
        ExprVec::try_from_be_bytes(&bytes).unwrap_err(),
        FromFieldVec::try_from_be_bytes(&bytes).unwrap_err(),
    ] {
        assert!(
            matches!(error, bebytes::BeBytesError::InsufficientData { .. }),
            "{error:?}"
        );
    }
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct Mixed {
    a: u8,
    b: u8,
    #[With(size(a * b + 1))]
    data: Vec<u8>,
    divisor: u8,
    #[With(size(a / divisor))]
    text: String,
    len: u64,
    #[FromField(len)]
    tail: Vec<u8>,
}

fn boundary_u64() -> impl Strategy<Value = u64> {
    prop_oneof![
        any::<u64>(),
        0u64..8,
        (u64::MAX - 16)..=u64::MAX,
        ((usize::MAX as u64) - 16)..=(usize::MAX as u64),
    ]
}

fn boundary_u8() -> impl Strategy<Value = u8> {
    prop_oneof![any::<u8>(), 0u8..3, 253u8..=255]
}

proptest! {
    #[test]
    fn boundary_lengths_never_panic(
        a in boundary_u8(),
        b in boundary_u8(),
        divisor in boundary_u8(),
        len in boundary_u64(),
        body in proptest::collection::vec(any::<u8>(), 0..32),
    ) {
        let mut bytes = vec![a, b];
        bytes.extend_from_slice(&body);
        bytes.push(divisor);
        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.extend_from_slice(&body);
        let _ = Mixed::try_from_be_bytes(&bytes);
        let _ = Mixed::try_from_le_bytes(&bytes);
        let _ = ExprVec::try_from_be_bytes(&bytes);
        let _ = FromFieldVec::try_from_be_bytes(&bytes);
        let _ = FromFieldString::try_from_le_bytes(&bytes);
        let _ = WideMul::try_from_be_bytes(&bytes);
        let _ = WideAdd::try_from_le_bytes(&bytes);
    }
}

proptest! {
    #[test]
    fn declared_length_matches_available_bytes(
        len in 0u8..32,
        body in proptest::collection::vec(any::<u8>(), 0..64),
    ) {
        let mut bytes = vec![len];
        bytes.extend_from_slice(&body);
        let parsed = ExprByte::try_from_be_bytes(&bytes);
        prop_assert_eq!(parsed.is_ok(), body.len() >= usize::from(len));
    }
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct ExprByte {
    len: u8,
    #[With(size(len))]
    data: Vec<u8>,
}
