use bebytes::BeBytes;
use proptest::prelude::*;

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct Inner {
    id: [u8; 2],
    #[bits(4)]
    hi: u8,
    #[bits(4)]
    lo: u8,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct FixedMix {
    head: u8,
    #[bits(3)]
    a: u8,
    #[bits(13)]
    b: u16,
    signed: i32,
    letter: char,
    flag: bool,
    wide: u128,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct Arrays {
    tag: u8,
    block: [u8; 6],
    maybe: Option<u16>,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct Counted {
    count: u8,
    #[FromField(count)]
    items: Vec<Inner>,
    tail: u8,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct Sized {
    #[With(size(3))]
    items: Vec<Inner>,
    #[With(size(2))]
    text: String,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct Trailing {
    head: u8,
    rest: Vec<u8>,
}

macro_rules! arbitrary_input {
    ($test:ident, $ty:ident) => {
        proptest! {
            #[test]
            fn $test(bytes in proptest::collection::vec(any::<u8>(), 0..64)) {
                if let Ok((value, consumed)) = $ty::try_from_be_bytes(&bytes) {
                    prop_assert!(consumed <= bytes.len());
                    let (again, _) = $ty::try_from_be_bytes(&value.to_be_bytes())
                        .map_err(|error| TestCaseError::fail(format!("{error:?}")))?;
                    prop_assert_eq!(again, value);
                }
                if let Ok((value, consumed)) = $ty::try_from_le_bytes(&bytes) {
                    prop_assert!(consumed <= bytes.len());
                    let (again, _) = $ty::try_from_le_bytes(&value.to_le_bytes())
                        .map_err(|error| TestCaseError::fail(format!("{error:?}")))?;
                    prop_assert_eq!(again, value);
                }
            }
        }
    };
}

arbitrary_input!(inner_never_panics, Inner);
arbitrary_input!(fixed_mix_never_panics, FixedMix);
arbitrary_input!(arrays_never_panics, Arrays);
arbitrary_input!(counted_never_panics, Counted);
arbitrary_input!(sized_never_panics, Sized);
arbitrary_input!(trailing_never_panics, Trailing);
