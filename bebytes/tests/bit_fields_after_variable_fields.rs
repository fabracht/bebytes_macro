use bebytes::BeBytes;

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct AfterString {
    #[With(size(3))]
    text: String,
    #[bits(4)]
    high: u8,
    #[bits(4)]
    low: u8,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct AfterVec {
    #[With(size(2))]
    data: Vec<u8>,
    #[bits(4)]
    high: u8,
    #[bits(4)]
    low: u8,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct AfterOption {
    maybe: Option<u16>,
    #[bits(4)]
    high: u8,
    #[bits(4)]
    low: u8,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct AfterArray {
    block: [u8; 3],
    #[bits(12)]
    word: u16,
    #[bits(4)]
    nibble: u8,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct AfterStringWide {
    #[With(size(3))]
    text: String,
    #[bits(12)]
    word: u16,
    #[bits(4)]
    nibble: u8,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct AfterVecWide {
    #[With(size(2))]
    data: Vec<u8>,
    #[bits(12)]
    word: u16,
    #[bits(4)]
    nibble: u8,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct AfterOptionWide {
    maybe: Option<u16>,
    #[bits(12)]
    word: u16,
    #[bits(4)]
    nibble: u8,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct AfterLongString {
    #[With(size(8))]
    text: String,
    #[bits(4)]
    high: u8,
    #[bits(4)]
    low: u8,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct AfterLongVec {
    #[With(size(9))]
    data: Vec<u8>,
    #[bits(4)]
    high: u8,
    #[bits(4)]
    low: u8,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct AfterLongArray {
    block: [u8; 9],
    #[bits(4)]
    high: u8,
    #[bits(4)]
    low: u8,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct AfterWideOption {
    maybe: Option<u128>,
    #[bits(4)]
    high: u8,
    #[bits(4)]
    low: u8,
}

fn check<T: BeBytes + PartialEq + core::fmt::Debug>(value: &T, big: &[u8], little: &[u8]) {
    assert_eq!(value.to_be_bytes(), big);
    assert_eq!(value.to_le_bytes(), little);
    let (from_big, big_len) = T::try_from_be_bytes(big).unwrap();
    assert_eq!(&from_big, value);
    assert_eq!(big_len, big.len());
    let (from_little, little_len) = T::try_from_le_bytes(little).unwrap();
    assert_eq!(&from_little, value);
    assert_eq!(little_len, little.len());
}

#[test]
fn nibbles_after_sized_string() {
    let value = AfterString {
        text: "abc".to_string(),
        high: 0x8,
        low: 0x9,
    };
    check(&value, &[b'a', b'b', b'c', 0x89], &[b'a', b'b', b'c', 0x98]);
}

#[test]
fn nibbles_after_sized_vec() {
    let value = AfterVec {
        data: vec![0x11, 0x22],
        high: 0x8,
        low: 0x9,
    };
    check(&value, &[0x11, 0x22, 0x89], &[0x11, 0x22, 0x98]);
}

#[test]
fn nibbles_after_option() {
    let value = AfterOption {
        maybe: Some(0x0102),
        high: 0x8,
        low: 0x9,
    };
    check(&value, &[1, 0x01, 0x02, 0x89], &[1, 0x02, 0x01, 0x98]);
}

#[test]
fn twelve_bit_word_after_array() {
    let value = AfterArray {
        block: [1, 2, 3],
        word: 0xABC,
        nibble: 0xD,
    };
    check(&value, &[1, 2, 3, 0xAB, 0xCD], &[1, 2, 3, 0xBC, 0xDA]);
}

#[test]
fn twelve_bit_word_after_sized_string() {
    let value = AfterStringWide {
        text: "abc".to_string(),
        word: 0xABC,
        nibble: 0xD,
    };
    check(
        &value,
        &[b'a', b'b', b'c', 0xAB, 0xCD],
        &[b'a', b'b', b'c', 0xBC, 0xDA],
    );
}

#[test]
fn twelve_bit_word_after_sized_vec() {
    let value = AfterVecWide {
        data: vec![0x11, 0x22],
        word: 0xABC,
        nibble: 0xD,
    };
    check(&value, &[0x11, 0x22, 0xAB, 0xCD], &[0x11, 0x22, 0xBC, 0xDA]);
}

#[test]
fn twelve_bit_word_after_option() {
    let value = AfterOptionWide {
        maybe: Some(0x0102),
        word: 0xABC,
        nibble: 0xD,
    };
    check(
        &value,
        &[1, 0x01, 0x02, 0xAB, 0xCD],
        &[1, 0x02, 0x01, 0xBC, 0xDA],
    );
}

#[test]
fn nibbles_after_long_string() {
    let value = AfterLongString {
        text: "abcdefgh".to_string(),
        high: 0x8,
        low: 0x9,
    };
    let mut be = b"abcdefgh".to_vec();
    be.push(0x89);
    let mut le = b"abcdefgh".to_vec();
    le.push(0x98);
    check(&value, &be, &le);
}

#[test]
fn nibbles_after_long_vec() {
    let value = AfterLongVec {
        data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
        high: 0x8,
        low: 0x9,
    };
    check(
        &value,
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0x89],
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0x98],
    );
}

#[test]
fn nibbles_after_long_array() {
    let value = AfterLongArray {
        block: [1, 2, 3, 4, 5, 6, 7, 8, 9],
        high: 0x8,
        low: 0x9,
    };
    check(
        &value,
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0x89],
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0x98],
    );
}

#[test]
fn nibbles_after_wide_option() {
    let value = AfterWideOption {
        maybe: Some(0x0102_0304_0506_0708_090A_0B0C_0D0E_0F10),
        high: 0x8,
        low: 0x9,
    };
    let be = [
        1, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
        0x0F, 0x10, 0x89,
    ];
    let le = [
        1, 0x10, 0x0F, 0x0E, 0x0D, 0x0C, 0x0B, 0x0A, 0x09, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03,
        0x02, 0x01, 0x98,
    ];
    check(&value, &be, &le);
}
