use bebytes::BeBytes;
use proptest::prelude::*;

struct Part {
    value: u128,
    width: usize,
}

fn push_bits(bytes: &mut [u8], position: &mut usize, value: u128, width: usize) {
    for shift in (0..width).rev() {
        if (value >> shift) & 1 == 1 {
            bytes[*position / 8] |= 0x80 >> (*position % 8);
        }
        *position += 1;
    }
}

fn reference_pack(parts: &[Part], little_endian: bool) -> Vec<u8> {
    let total_bits: usize = parts.iter().map(|part| part.width).sum();
    let mut bytes = vec![0u8; total_bits.div_ceil(8)];
    let mut position = 0;
    for part in parts {
        if little_endian {
            for bit in 0..part.width {
                if (part.value >> bit) & 1 == 1 {
                    bytes[(position + bit) / 8] |= 1 << ((position + bit) % 8);
                }
            }
            position += part.width;
        } else {
            push_bits(&mut bytes, &mut position, part.value, part.width);
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

fn from_pattern<const N: usize>(pattern: u128) -> [u8; N] {
    let mut out = [0u8; N];
    out.copy_from_slice(&pattern.to_be_bytes()[16 - N..]);
    out
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
    ($be_test:ident, $le_test:ident, $name:ident { $( $(#[bits($bits:literal)])? $field:ident: $ty:ident ),+ $(,)? }) => {
        #[derive(BeBytes, Debug, PartialEq)]
        struct $name {
            $( $(#[bits($bits)])? $field: $ty ),+
        }

        impl $name {
            fn from_seeds(seeds: [u128; 16]) -> (Self, Vec<Part>) {
                let mut remaining = seeds.into_iter();
                $(
                    let $field = masked(
                        remaining.next().unwrap_or_default(),
                        field_width!($ty $(, $bits)?),
                    );
                )+
                let parts = vec![
                    $( Part {
                        value: $field,
                        width: field_width!($ty $(, $bits)?),
                    } ),+
                ];
                let value = $name {
                    $(
                        $field: <$ty>::from_be_bytes(from_pattern($field))
                    ),+
                };
                (value, parts)
            }
        }

        proptest! {
            #[test]
            fn $be_test(seeds in proptest::array::uniform16(any::<u128>())) {
                let (value, parts) = $name::from_seeds(seeds);
                let expected = reference_pack(&parts, false);

                prop_assert_eq!(value.to_be_bytes(), expected.clone());

                let mut buffer = bebytes::BytesMut::with_capacity(0);
                value
                    .encode_be_to(&mut buffer)
                    .map_err(|error| TestCaseError::fail(format!("{error:?}")))?;
                prop_assert_eq!(buffer.to_vec(), expected.clone());

                let (parsed, consumed) = $name::try_from_be_bytes(&expected)
                    .map_err(|error| TestCaseError::fail(format!("{error:?}")))?;
                prop_assert_eq!(parsed, value);
                prop_assert_eq!(consumed, expected.len());
            }

            #[test]
            fn $le_test(seeds in proptest::array::uniform16(any::<u128>())) {
                let (value, parts) = $name::from_seeds(seeds);
                let expected = reference_pack(&parts, true);

                prop_assert_eq!(value.to_le_bytes(), expected.clone());

                let mut buffer = bebytes::BytesMut::with_capacity(0);
                value
                    .encode_le_to(&mut buffer)
                    .map_err(|error| TestCaseError::fail(format!("{error:?}")))?;
                prop_assert_eq!(buffer.to_vec(), expected.clone());

                let (parsed, consumed) = $name::try_from_le_bytes(&expected)
                    .map_err(|error| TestCaseError::fail(format!("{error:?}")))?;
                prop_assert_eq!(parsed, value);
                prop_assert_eq!(consumed, expected.len());
            }
        }
    };
}

oracle_layout!(
    byte_then_two_nibbles_then_word,
    byte_then_two_nibbles_then_word_le,
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
    nibble_twelve_then_byte_le,
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
    interleaved_groups_le,
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
    wide_groups_le,
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

oracle_layout!(
    signed_whole_fields,
    signed_whole_fields_le,
    SignedWholeFields {
        first: i8,
        #[bits(4)]
        high: u8,
        #[bits(4)]
        low: u8,
        second: i16,
        third: i32,
        fourth: i64,
        fifth: i128,
    }
);

oracle_layout!(
    aligned_partial_and_whole,
    aligned_partial_and_whole_le,
    AlignedPartialAndWhole {
        #[bits(16)]
        whole_word: u16,
        #[bits(12)]
        partial_word: u16,
        #[bits(4)]
        nibble: u8,
        #[bits(24)]
        partial_long: u32,
        #[bits(8)]
        whole_byte: u8,
        #[bits(32)]
        whole_long: u32,
    }
);

oracle_layout!(
    unaligned_whole_width,
    unaligned_whole_width_le,
    UnalignedWholeWidth {
        #[bits(4)]
        nibble: u8,
        #[bits(16)]
        word_at_nibble: u16,
        #[bits(4)]
        trailing: u8,
        #[bits(32)]
        long_aligned: u32,
        #[bits(3)]
        three: u8,
        #[bits(5)]
        five: u8,
        #[bits(64)]
        wide_aligned: u64,
    }
);

#[test]
fn documented_bit_order_example() {
    let header = ByteNibblesWord {
        head: 0x01,
        high: 0x8,
        low: 0x9,
        tail: 0x0A0B,
    };

    assert_eq!(header.to_be_bytes(), [0x01, 0x89, 0x0A, 0x0B]);
    assert_eq!(header.to_le_bytes(), [0x01, 0x98, 0x0B, 0x0A]);
}

oracle_layout!(
    small_fields_on_wide_types,
    small_fields_on_wide_types_le,
    SmallFieldsOnWideTypes {
        #[bits(3)]
        a: u16,
        #[bits(5)]
        b: u32,
        #[bits(8)]
        c: u64,
        #[bits(2)]
        d: u128,
        #[bits(6)]
        e: u64,
        #[bits(8)]
        f: u8,
        #[bits(8)]
        g: u16,
        #[bits(8)]
        h: u128,
    }
);
