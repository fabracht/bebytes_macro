use bebytes::BeBytes;

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

#[test]
fn test_nested_structs() {
    let dummy_struct = DummyStruct {
        dummy0: [1, 2],
        dummy1: 1,
        dummy2: 100,
    };

    let nested_struct = NestedStruct {
        s_bit: 1,
        z_bit: 0,
        scale: 31,
        dummy_struct: dummy_struct.clone(),
    };

    // Test serialization
    let bytes = nested_struct.to_be_bytes();
    assert_eq!(bytes.len(), 4); // 1 byte for bits + 3 bytes for DummyStruct

    // Test deserialization
    let (result, consumed) = NestedStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(result, nested_struct);
    assert_eq!(consumed, 4);

    // Test field access
    assert_eq!(result.s_bit, 1);
    assert_eq!(result.z_bit, 0);
    assert_eq!(result.scale, 31);
    assert_eq!(result.dummy_struct.dummy0, [1, 2]);
    assert_eq!(result.dummy_struct.dummy1, 1);
    assert_eq!(result.dummy_struct.dummy2, 100);
}
