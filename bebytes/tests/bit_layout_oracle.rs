use bebytes::BeBytes;
use proptest::prelude::*;

fn reference_pack(parts: &[(u128, usize)]) -> Vec<u8> {
    let total_bits: usize = parts.iter().map(|&(_, width)| width).sum();
    let mut bytes = vec![0u8; total_bits.div_ceil(8)];
    let mut position = 0;
    for &(value, width) in parts {
        for shift in (0..width).rev() {
            if (value >> shift) & 1 == 1 {
                bytes[position / 8] |= 0x80 >> (position % 8);
            }
            position += 1;
        }
    }
    bytes
}

fn masked(seed: u128, width: usize) -> u128 {
    if width >= 128 {
        seed
    } else {
        seed & ((1u128 << width) - 1)
    }
}

macro_rules! field_width {
    ($ty:ident) => {
        8 * core::mem::size_of::<$ty>()
    };
    ($ty:ident, $bits:literal) => {
        $bits
    };
}

macro_rules! oracle_layout {
    ($test:ident, $name:ident { $( $(#[bits($bits:literal)])? $field:ident: $ty:ident ),+ $(,)? }) => {
        #[derive(BeBytes, Debug, PartialEq)]
        struct $name {
            $( $(#[bits($bits)])? $field: $ty ),+
        }

        proptest! {
            #[test]
            fn $test(seeds in proptest::array::uniform16(any::<u128>())) {
                let mut remaining = seeds.into_iter();
                $(
                    let $field = masked(
                        remaining.next().unwrap_or_default(),
                        field_width!($ty $(, $bits)?),
                    );
                )+
                let expected = reference_pack(&[
                    $( ($field, field_width!($ty $(, $bits)?)) ),+
                ]);
                let value = $name {
                    $(
                        $field: <$ty>::try_from($field)
                            .map_err(|error| TestCaseError::fail(format!("{error:?}")))?
                    ),+
                };

                prop_assert_eq!(value.to_be_bytes(), expected.clone());

                let (parsed, consumed) = $name::try_from_be_bytes(&expected)
                    .map_err(|error| TestCaseError::fail(format!("{error:?}")))?;
                prop_assert_eq!(parsed, value);
                prop_assert_eq!(consumed, expected.len());
            }
        }
    };
}

oracle_layout!(
    byte_then_two_nibbles_then_word,
    ByteNibblesWord {
        head: u8,
        #[bits(4)]
        high: u8,
        #[bits(4)]
        low: u8,
        tail: u16,
    }
);

oracle_layout!(
    nibble_twelve_then_byte,
    NibbleTwelveByte {
        #[bits(4)]
        nibble: u8,
        #[bits(12)]
        twelve: u16,
        regular: u8,
    }
);

oracle_layout!(
    interleaved_groups,
    InterleavedGroups {
        #[bits(3)]
        a: u8,
        #[bits(5)]
        b: u8,
        middle: u8,
        #[bits(1)]
        c: u8,
        #[bits(7)]
        d: u8,
        tail: u16,
    }
);

oracle_layout!(
    wide_groups,
    WideGroups {
        #[bits(20)]
        wide: u32,
        #[bits(4)]
        nibble: u8,
        after: u8,
        #[bits(10)]
        ten: u16,
        #[bits(6)]
        six: u16,
        last: u32,
    }
);
