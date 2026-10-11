use bebytes::{BeBytes, BeBytesError};

#[derive(BeBytes, Debug, PartialEq)]
struct Nibbles {
    #[bits(4)]
    a: u8,
    #[bits(4)]
    b: u8,
    #[bits(4)]
    c: u8,
    #[bits(4)]
    d: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct SpanningBits {
    #[bits(3)]
    a: u8,
    #[bits(7)]
    b: u8,
    #[bits(6)]
    c: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct WithArray {
    tag: u8,
    id: [u8; 4],
}

#[derive(BeBytes, Debug, PartialEq)]
struct StringThenArray {
    len: u8,
    #[FromField(len)]
    name: String,
    id: [u8; 4],
}

#[derive(BeBytes, Debug, PartialEq)]
struct SizedBytes {
    len: u8,
    #[FromField(len)]
    data: Vec<u8>,
    tail: u8,
}

fn assert_every_cut_fails<T: BeBytes>(full: &[u8]) {
    for cut in 0..full.len() {
        let truncated = &full[..cut];
        assert!(T::try_from_be_bytes(truncated).is_err(), "be cut at {cut}");
        assert!(T::try_from_le_bytes(truncated).is_err(), "le cut at {cut}");
    }
    assert!(T::try_from_be_bytes(full).is_ok());
    assert!(T::try_from_le_bytes(full).is_ok());
}

#[test]
fn multi_byte_bit_field_run_rejects_truncation() {
    let full = Nibbles {
        a: 1,
        b: 2,
        c: 3,
        d: 4,
    }
    .to_be_bytes();
    assert_every_cut_fails::<Nibbles>(&full);
}

#[test]
fn bit_field_spanning_bytes_rejects_truncation() {
    let full = SpanningBits {
        a: 5,
        b: 100,
        c: 33,
    }
    .to_be_bytes();
    assert_every_cut_fails::<SpanningBits>(&full);
}

#[test]
fn byte_array_rejects_truncation() {
    let full = WithArray {
        tag: 7,
        id: [1, 2, 3, 4],
    }
    .to_be_bytes();
    assert_every_cut_fails::<WithArray>(&full);
    assert_eq!(
        WithArray::try_from_be_bytes(&[7, 1]),
        Err(BeBytesError::InsufficientData {
            expected: 4,
            actual: 1,
        })
    );
}

#[test]
fn array_after_dynamic_string_rejects_truncation() {
    let full = StringThenArray {
        len: 3,
        name: "abc".to_string(),
        id: [9, 8, 7, 6],
    }
    .to_be_bytes();
    assert_every_cut_fails::<StringThenArray>(&full);
}

#[test]
fn field_sized_byte_vector_rejects_truncation() {
    let full = SizedBytes {
        len: 3,
        data: vec![1, 2, 3],
        tail: 4,
    }
    .to_be_bytes();
    assert_every_cut_fails::<SizedBytes>(&full);
    assert_eq!(
        SizedBytes::try_from_be_bytes(&[5, 1, 2]),
        Err(BeBytesError::InsufficientData {
            expected: 5,
            actual: 2,
        })
    );
}
