#![allow(clippy::assign_op_pattern)]

use bebytes::BeBytes;

fn main() {
    // Test both endianness formats
    test_both_endianness();

    let client_setup_response = ArrayedStruct::new(Modes::new(0), [1; 1], [2; 2], [3; 3]);
    let bytes = client_setup_response.to_be_bytes();
    println!("Bytes len: {}", bytes.len());
    for byte in &bytes {
        print!("{:08b} ", byte);
    }
    let smalls_father1 = ArrayedStruct::try_from_be_bytes(&bytes);
    println!("\nSmallStrucFather: {:?}", smalls_father1);

    let small_struct = SmallStruct { small: 3 };
    let smalls_father = SmallStructFather {
        small_struct,
        num1: 5,
    };
    let bytes = smalls_father.to_be_bytes();
    println!("Bytes len: {}", bytes.len());
    for byte in &bytes {
        print!("{:08b} ", byte);
    }
    let smalls_father1 = SmallStructFather::try_from_be_bytes(&bytes);
    println!("\nSmallStrucFather: {:?}", smalls_father1);

    let error_estimate = ErrorEstimateMini {
        s_bit: 1,
        z_bit: 0,
        scale: 63,
        multiplier: 3,
    };
    let bytes = error_estimate.to_be_bytes();
    println!("Bytes len: {}", bytes.len());
    for byte in &bytes {
        print!("{:08b} ", byte);
    }
    let error = ErrorEstimateMini::try_from_be_bytes(&bytes);
    println!("\nError: {:?}", error);
    assert_eq!(error_estimate, error.unwrap().0);
    let error_estimate = ErrorEstimate {
        s_bit: 1,
        z_bit: 0,
        scale: 63,
        dummy_struct: DummyStruct {
            dummy1: 1,
            dummy2: 2,
            dummy0: [3, 4],
        },
    };
    let bytes = error_estimate.to_be_bytes();
    println!("Bytes len: {}", bytes.len());
    for byte in &bytes {
        print!("{:08b} ", byte);
    }

    let error = ErrorEstimate::try_from_be_bytes(&bytes);
    println!("\nError: {:?}", error);
    assert_eq!(error_estimate, error.unwrap().0);
    let dummy = DummyStruct {
        dummy0: [0, 2],
        dummy1: 1,
        dummy2: 2,
    };
    let dummy_bytes = dummy.to_be_bytes();

    let re_dummy = DummyStruct::try_from_be_bytes(&dummy_bytes);
    println!("\ndummy error {:?}", re_dummy);
    assert_eq!(dummy, re_dummy.unwrap().0);
    let nested = NestedStruct::new(dummy, None, error_estimate);
    let nested_bytes = nested.to_be_bytes();
    println!("Nested bytes:");
    for byte in &nested_bytes {
        print!("{:08b} ", byte);
    }
    let dummy_enum = DummyEnum::ServerStart;
    let dummy_enum_bytes = dummy_enum.to_be_bytes();
    let re_dummy_enum = DummyEnum::try_from_be_bytes(&dummy_enum_bytes);
    assert_eq!(dummy_enum, re_dummy_enum.unwrap().0);

    let u_8 = U8 {
        first: 1,
        second: 2,
        third: 3,
        fourth: 4,
    };
    let u_8_bytes = u_8.to_be_bytes();
    println!("{:?}", u_8_bytes);
    let re_u_8 = U8::try_from_be_bytes(&u_8_bytes);
    println!("{:?}", re_u_8);
    assert_eq!(u_8, re_u_8.unwrap().0);

    let u_16 = U16 {
        first: 1,
        second: 16383,
        fourth: 0,
    };
    let u_16_bytes = u_16.to_be_bytes();

    println!("{:?}", u_16_bytes);
    let re_u_16 = U16::try_from_be_bytes(&u_16_bytes);
    println!("{:?}", re_u_16);
    assert_eq!(u_16, re_u_16.unwrap().0);

    let u_32 = U32 {
        first: 1,
        second: 32383,
        fourth: 1,
    };
    let u_32_bytes = u_32.to_be_bytes();

    println!("{:?}", u_32_bytes);
    let re_u_32 = U32::try_from_be_bytes(&u_32_bytes);
    println!("{:?}", re_u_32);
    assert_eq!(u_32, re_u_32.unwrap().0);

    let optional = Optional {
        optional_number: Some(5),
    };
    let optional_bytes = optional.to_be_bytes();
    println!("Optional Some: {:?}", optional_bytes);
    let none_optional = Optional {
        optional_number: None,
    };
    let none_optional_bytes = none_optional.to_be_bytes();
    println!("Optional None: {:?}", none_optional_bytes);
    let with_tailing_vec = WithTailingVec {
        pre_tail: 2,
        tail: vec![2, 3],
        post_tail: 3,
    };
    let with_tailing_vec_bytes = with_tailing_vec.to_be_bytes();
    println!("WithTailingVec: {:?}", with_tailing_vec_bytes);
    let re_with_tailing_vec = WithTailingVec::try_from_be_bytes(&with_tailing_vec_bytes);
    println!("ReWithTailingVec: {:?}", re_with_tailing_vec);
    assert_eq!(with_tailing_vec, re_with_tailing_vec.unwrap().0);

    let with_size_struct = WithSizeStruct {
        innocent: 1,
        real_tail: vec![2, 3, 4],
    };
    let with_size_struct_bytes = with_size_struct.to_be_bytes();
    println!("WithSizeStruct: {:?}", with_size_struct_bytes);
    let re_with_size_struct = WithSizeStruct::try_from_be_bytes(&with_size_struct_bytes);
    println!("ReWithSizeStruct: {:?}", re_with_size_struct);
    assert_eq!(with_size_struct, re_with_size_struct.unwrap().0);

    let innocent_struct = InnocentStruct {
        innocent: 1,
        real_tail: vec![4, 5],
    };
    let innocent_struct_bytes = innocent_struct.to_be_bytes();
    println!("InnocentStruct: {:?}", innocent_struct_bytes);
    let re_innocent_struct = InnocentStruct::try_from_be_bytes(&innocent_struct_bytes);
    println!("ReInnocentStruct: {:?}", re_innocent_struct);
    assert_eq!(innocent_struct, re_innocent_struct.unwrap().0);

    let complete_func = CompleteFunctionality::new(
        1,
        2,
        3,
        vec![6, 7, 8],
        4,
        vec![5, 4, 3, 2],
        U16::new(1, 2, 1),
        ArrayedStruct::new(Modes::new(1), [2; 1], [3; 2], [4; 3]),
        DummyEnum::ServerStart,
        Some(5),
        Modes::new(3),
        WithTailingVec {
            pre_tail: 4,
            tail: vec![1, 2, 3, 4],
            post_tail: 5,
        },
        U32::new(1, 57, 1),
        vec![1, 2, 3, 4, 5],
    );
    let complete_func_bytes = complete_func.to_be_bytes();
    println!("CompleteFunctionality: {:?}", complete_func_bytes);
    let re_complete_func = CompleteFunctionality::try_from_be_bytes(&complete_func_bytes);
    println!("ReCompleteFunctionality: {:?}", re_complete_func);
    assert_eq!(complete_func, re_complete_func.unwrap().0);
    let u_64 = U64 {
        first: 1,
        second: (1 << 62) - 1,
        fourth: 1,
    };
    let u_64_bytes = u_64.to_be_bytes();
    println!("{:?}", u_64_bytes);
    let re_u_64 = U64::try_from_be_bytes(&u_64_bytes);
    println!("{:?}", re_u_64);
    assert_eq!(u_64, re_u_64.unwrap().0);

    let u_128 = U128 {
        first: 1,
        second: 1,
        fourth: 1,
    };
    let u_128_bytes = u_128.to_be_bytes();
    println!("{:?}", u_128_bytes);
    let re_u_128 = U128::try_from_be_bytes(&u_128_bytes);
    println!("{:?}", re_u_128);
    assert_eq!(u_128, re_u_128.unwrap().0);

    let i_8 = I8 {
        first: 1,
        second: 1,
        fourth: 1,
    };
    let i_8_bytes = i_8.to_be_bytes();
    println!("{:?}", i_8_bytes);
    let re_i_8 = I8::try_from_be_bytes(&i_8_bytes);
    println!("{:?}", re_i_8);
    assert_eq!(i_8, re_i_8.unwrap().0);
}

// Test that both endianness formats work correctly
fn test_both_endianness() {
    println!("\n=== TESTING BOTH ENDIANNESS FORMATS ===\n");

    // Test with a simple struct
    let test_struct = U16 {
        first: 1,
        second: 16383,
        fourth: 0,
    };

    // Convert to big-endian
    let be_bytes = test_struct.to_be_bytes();
    println!("Big-endian bytes: {:?}", be_bytes);

    // Convert to little-endian
    let le_bytes = test_struct.to_le_bytes();
    println!("Little-endian bytes: {:?}", le_bytes);

    // They should be different
    assert_ne!(
        be_bytes, le_bytes,
        "Big-endian and little-endian representations should differ"
    );

    // Parse from big-endian
    let (from_be, be_len) = U16::try_from_be_bytes(&be_bytes).unwrap();
    println!("Parsed from BE: {:?}, len: {}", from_be, be_len);
    assert_eq!(test_struct, from_be);

    // Parse from little-endian
    let (from_le, le_len) = U16::try_from_le_bytes(&le_bytes).unwrap();
    println!("Parsed from LE: {:?}, len: {}", from_le, le_len);
    assert_eq!(test_struct, from_le);

    // Parsing big-endian as little-endian should yield incorrect results
    if let Ok((wrong_endian, _)) = U16::try_from_le_bytes(&be_bytes) {
        assert_ne!(
            test_struct, wrong_endian,
            "Parsing BE bytes as LE should give different results"
        );
        println!("BE bytes parsed as LE (incorrectly): {:?}", wrong_endian);
    }

    // Test with a medium complexity struct (U32)
    let test_u32 = U32 {
        first: 1,
        second: 32383,
        fourth: 1,
    };

    // Test big-endian serialization and deserialization
    let u32_be = test_u32.to_be_bytes();
    println!("U32 BE bytes: {:?}", u32_be);
    let (parsed_u32_be, _) = U32::try_from_be_bytes(&u32_be).unwrap();
    assert_eq!(test_u32, parsed_u32_be);

    // Test little-endian serialization and deserialization
    let u32_le = test_u32.to_le_bytes();
    println!("U32 LE bytes: {:?}", u32_le);
    let (parsed_u32_le, _) = U32::try_from_le_bytes(&u32_le).unwrap();
    assert_eq!(test_u32, parsed_u32_le);

    // They should be different representations
    assert_ne!(u32_be, u32_le);

    println!("Both endianness formats work correctly!\n");
}

#[derive(BeBytes, Debug, PartialEq)]
struct I8 {
    #[U8(size(1), pos(0))]
    first: u8,
    #[U8(size(6), pos(1))]
    second: i8,
    #[U8(size(1), pos(7))]
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct F32 {
    first: u8,
    second: f32,
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct U128 {
    #[U8(size(1), pos(0))]
    first: u8,
    #[U8(size(126), pos(1))]
    second: u128,
    #[U8(size(1), pos(127))]
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct U64 {
    #[U8(size(1), pos(0))]
    first: u8,
    #[U8(size(62), pos(1))]
    second: u64,
    #[U8(size(1), pos(63))]
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct CompleteFunctionality {
    #[U8(size(1), pos(0))]
    first: u8,
    #[U8(size(3), pos(1))]
    second: u8,
    #[U8(size(4), pos(4))]
    third: u8,
    #[With(size(3))]
    with_size: Vec<u8>,
    fourth: u8,
    #[FromField(fourth)]
    body: Vec<u8>,
    u_16: U16,
    arrayed: ArrayedStruct,
    dummy_enum: DummyEnum,
    optional: Option<i32>,
    modes: Modes,
    vecty: WithTailingVec,
    u_32: U32,
    rattle: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct U8 {
    #[U8(size(1), pos(0))]
    first: u8,
    #[U8(size(3), pos(1))]
    second: u8,
    #[U8(size(4), pos(4))]
    third: u8,
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
struct U16 {
    #[U8(size(1), pos(0))]
    first: u8,
    #[U8(size(14), pos(1))]
    second: u16,
    #[U8(size(1), pos(15))]
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct U32 {
    #[U8(size(1), pos(0))]
    first: u8,
    #[U8(size(30), pos(1))]
    second: u32,
    #[U8(size(1), pos(31))]
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
pub enum DummyEnum {
    SetupResponse,
    ServerStart,
    SetupRequest,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
pub struct DummyStruct {
    pub dummy0: [u8; 2],
    #[U8(size(1), pos(0))]
    pub dummy1: u8,
    #[U8(size(7), pos(1))]
    pub dummy2: u8,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
pub struct ErrorEstimate {
    #[U8(size(1), pos(0))]
    pub s_bit: u8,
    #[U8(size(1), pos(1))]
    pub z_bit: u8,
    #[U8(size(6), pos(2))]
    pub scale: u8,
    pub dummy_struct: DummyStruct,
}

#[derive(BeBytes, Debug, PartialEq)]
pub struct ErrorEstimateMini {
    #[U8(size(1), pos(0))]
    pub s_bit: u8,
    #[U8(size(1), pos(1))]
    pub z_bit: u8,
    #[U8(size(6), pos(2))]
    pub scale: u8,
    pub multiplier: u32,
}

#[derive(BeBytes, Debug, PartialEq)]
pub struct NestedStruct {
    pub dummy_struct: DummyStruct,
    pub optional_number: Option<i32>,
    pub error_estimate: ErrorEstimate,
}

#[derive(BeBytes, Debug, PartialEq)]
pub struct Optional {
    pub optional_number: Option<i32>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
pub struct SmallStruct {
    pub small: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
pub struct SmallStructFather {
    small_struct: SmallStruct,
    num1: u32,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
pub struct ArrayedStruct {
    pub mode: Modes,
    pub key_id: [u8; 1],
    pub token: [u8; 2],
    pub client_iv: [u8; 3],
}

#[derive(BeBytes, Debug, PartialEq, Clone, Default)]
pub struct Modes {
    pub bits: u8,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Mode {
    Closed = 0b0000,
    Unauthenticated = 0b0001,
    Authenticated = 0b0010,
    Encrypted = 0b0100,
}

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct WithTailingVec {
    pre_tail: u8,
    #[FromField(pre_tail)]
    tail: Vec<u8>,
    post_tail: u8,
}

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct InnocentStruct {
    innocent: u8,
    real_tail: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct WithSizeStruct {
    innocent: u8,
    #[With(size(3))]
    real_tail: Vec<u8>,
}
