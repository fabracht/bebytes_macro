use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
enum TestEnum {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
}

#[derive(BeBytes, Debug, PartialEq)]
struct TestStruct {
    #[bits(4)]
    field1: u8,
    #[bits()]
    // Testing empty parentheses for auto-sizing - should use TestEnum::__BEBYTES_MIN_BITS (2 bits)
    field2: TestEnum,
    #[bits(2)]
    field3: u8,
}

#[test]
fn test_bare_bits_attribute() {
    let original = TestStruct {
        field1: 0x0F,
        field2: TestEnum::C,
        field3: 0x03,
    };

    let bytes = original.to_be_bytes();
    println!("Bytes: {:?}", bytes);
    
    // Test roundtrip: deserialize and verify equality
    let (deserialized, _) = TestStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(original, deserialized);
}
