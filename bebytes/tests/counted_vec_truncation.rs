use bebytes::{BeBytes, BeBytesError};

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct Inner {
    id: [u8; 4],
    #[bits(4)]
    hi: u8,
    #[bits(4)]
    lo: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct CountedLast {
    count: u8,
    #[FromField(count)]
    items: Vec<Inner>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct CountedMiddle {
    count: u8,
    #[FromField(count)]
    items: Vec<Inner>,
    tail: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct CountedThenRest {
    count: u8,
    #[FromField(count)]
    items: Vec<Inner>,
    rest: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct SizedLast {
    #[With(size(2))]
    items: Vec<Inner>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct SizedMiddle {
    #[With(size(2))]
    items: Vec<Inner>,
    tail: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct UnsizedLast {
    items: Vec<Inner>,
}

fn item(seed: u8) -> [u8; 5] {
    [seed, seed, seed, seed, 0x56]
}

fn inner(seed: u8) -> Inner {
    Inner {
        id: [seed; 4],
        hi: 5,
        lo: 6,
    }
}

fn is_insufficient<T: core::fmt::Debug>(result: Result<T, BeBytesError>) -> bool {
    matches!(result, Err(BeBytesError::InsufficientData { .. }))
}

#[test]
fn counted_last_rejects_truncated_first_item() {
    assert!(is_insufficient(CountedLast::try_from_be_bytes(&[
        1, 1, 2, 3, 4
    ])));
}

#[test]
fn counted_last_rejects_truncated_second_item() {
    let mut bytes = vec![2];
    bytes.extend_from_slice(&item(1));
    bytes.extend_from_slice(&[9, 9]);
    assert!(is_insufficient(CountedLast::try_from_be_bytes(&bytes)));
}

#[test]
fn counted_last_rejects_missing_items() {
    assert!(is_insufficient(CountedLast::try_from_be_bytes(&[2])));
}

#[test]
fn counted_last_honours_count_below_available_items() {
    let mut bytes = vec![1];
    bytes.extend_from_slice(&item(1));
    bytes.extend_from_slice(&item(2));
    let (parsed, consumed) = CountedLast::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(
        parsed,
        CountedLast {
            count: 1,
            items: vec![inner(1)]
        }
    );
    assert_eq!(consumed, 6);
}

#[test]
fn counted_last_parses_complete_input() {
    let mut bytes = vec![2];
    bytes.extend_from_slice(&item(1));
    bytes.extend_from_slice(&item(2));
    let (parsed, consumed) = CountedLast::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.items, vec![inner(1), inner(2)]);
    assert_eq!(consumed, 11);
}

#[test]
fn counted_middle_rejects_truncated_items() {
    let mut bytes = vec![2];
    bytes.extend_from_slice(&item(1));
    assert!(is_insufficient(CountedMiddle::try_from_be_bytes(&bytes)));
}

#[test]
fn counted_middle_parses_complete_input() {
    let mut bytes = vec![2];
    bytes.extend_from_slice(&item(1));
    bytes.extend_from_slice(&item(2));
    bytes.push(7);
    let (parsed, consumed) = CountedMiddle::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.items, vec![inner(1), inner(2)]);
    assert_eq!(parsed.tail, 7);
    assert_eq!(consumed, 12);
}

#[test]
fn counted_then_rest_rejects_missing_items() {
    let mut bytes = vec![2];
    bytes.extend_from_slice(&item(1));
    assert!(is_insufficient(CountedThenRest::try_from_be_bytes(&bytes)));
}

#[test]
fn counted_then_rest_keeps_trailing_bytes() {
    let mut bytes = vec![1];
    bytes.extend_from_slice(&item(1));
    bytes.extend_from_slice(&[7, 8]);
    let (parsed, consumed) = CountedThenRest::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.items, vec![inner(1)]);
    assert_eq!(parsed.rest, vec![7, 8]);
    assert_eq!(consumed, 8);
}

#[test]
fn sized_last_rejects_truncated_items() {
    let mut bytes = item(1).to_vec();
    bytes.extend_from_slice(&[9, 9]);
    assert!(is_insufficient(SizedLast::try_from_be_bytes(&bytes)));
}

#[test]
fn sized_last_honours_size_below_available_items() {
    let mut bytes = item(1).to_vec();
    bytes.extend_from_slice(&item(2));
    bytes.extend_from_slice(&item(3));
    let (parsed, consumed) = SizedLast::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.items, vec![inner(1), inner(2)]);
    assert_eq!(consumed, 10);
}

#[test]
fn sized_middle_rejects_truncated_items() {
    assert!(is_insufficient(SizedMiddle::try_from_be_bytes(&item(1))));
}

#[test]
fn unsized_last_consumes_all_complete_items() {
    let mut bytes = item(1).to_vec();
    bytes.extend_from_slice(&item(2));
    let (parsed, consumed) = UnsizedLast::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.items, vec![inner(1), inner(2)]);
    assert_eq!(consumed, 10);
}

#[test]
fn unsized_last_stops_at_first_incomplete_item() {
    let mut bytes = item(1).to_vec();
    bytes.extend_from_slice(&[9, 9]);
    let (parsed, consumed) = UnsizedLast::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(parsed.items, vec![inner(1)]);
    assert_eq!(consumed, 5);
}
