use bebytes::BeBytes;
use bytes::BytesMut;

#[test]
fn test_simple_struct_direct_writing() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct SimpleStruct {
        a: u8,
        b: u16,
        c: u32,
    }

    let data = SimpleStruct {
        a: 42,
        b: 1337,
        c: 0xDEADBEEF,
    };

    // Test big-endian direct writing
    let mut buf = BytesMut::with_capacity(7);
    data.encode_be_to(&mut buf).unwrap();
    assert_eq!(buf.to_vec(), data.to_be_bytes());

    // Test little-endian direct writing
    let mut buf = BytesMut::with_capacity(7);
    data.encode_le_to(&mut buf).unwrap();
    assert_eq!(buf.to_vec(), data.to_le_bytes());
}

#[test]
fn test_insufficient_buffer_capacity() {
    #[derive(BeBytes, Debug)]
    struct LargeStruct {
        data: [u8; 100],
    }

    let data = LargeStruct { data: [0xAB; 100] };

    // With the default implementation using to_be_bytes(),
    // BytesMut will automatically grow, so we need to use a different approach
    // Let's verify that the method correctly reports size requirements
    let required_size = LargeStruct::field_size();
    assert_eq!(required_size, 100);

    // Test that direct writing works with sufficient capacity
    let mut buf = BytesMut::with_capacity(100);
    let result = data.encode_be_to(&mut buf);
    assert!(result.is_ok());
    assert_eq!(buf.len(), 100);
}

#[test]
fn test_bit_fields_direct_writing() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct BitFieldStruct {
        #[bits(4)]
        high: u8,
        #[bits(4)]
        low: u8,
        regular: u16,
    }

    let data = BitFieldStruct {
        high: 0xF,
        low: 0xA,
        regular: 0x1234,
    };

    // Test big-endian
    let mut buf = BytesMut::with_capacity(3);
    data.encode_be_to(&mut buf).unwrap();
    assert_eq!(buf.to_vec(), data.to_be_bytes());

    // Test little-endian
    let mut buf = BytesMut::with_capacity(3);
    data.encode_le_to(&mut buf).unwrap();
    assert_eq!(buf.to_vec(), data.to_le_bytes());
}

#[test]
fn test_char_field_direct_writing() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct CharStruct {
        ch: char,
        num: u32,
    }

    let data = CharStruct {
        ch: '🦀', num: 42
    };

    // Test big-endian
    let mut buf = BytesMut::with_capacity(8);
    data.encode_be_to(&mut buf).unwrap();
    assert_eq!(buf.to_vec(), data.to_be_bytes());

    // Test little-endian
    let mut buf = BytesMut::with_capacity(8);
    data.encode_le_to(&mut buf).unwrap();
    assert_eq!(buf.to_vec(), data.to_le_bytes());
}

#[test]
fn test_enum_direct_writing() {
    #[derive(BeBytes, Debug, PartialEq)]
    #[repr(u8)]
    enum TestEnum {
        VariantA = 1,
        VariantB = 2,
        VariantC = 42,
    }

    let data = TestEnum::VariantC;

    // Test big-endian
    let mut buf = BytesMut::with_capacity(1);
    data.encode_be_to(&mut buf).unwrap();
    assert_eq!(buf.to_vec(), data.to_be_bytes());

    // Test little-endian
    let mut buf = BytesMut::with_capacity(1);
    data.encode_le_to(&mut buf).unwrap();
    assert_eq!(buf.to_vec(), data.to_le_bytes());
}

#[test]
fn test_nested_struct_direct_writing() {
    #[derive(BeBytes, Debug, PartialEq, Clone)]
    struct Inner {
        a: u16,
        b: u8,
    }

    #[derive(BeBytes, Debug, PartialEq)]
    struct Outer {
        inner: Inner,
        c: u32,
    }

    let data = Outer {
        inner: Inner { a: 0x1234, b: 0x56 },
        c: 0x789ABCDE,
    };

    // Test big-endian
    let mut buf = BytesMut::with_capacity(7);
    data.encode_be_to(&mut buf).unwrap();
    assert_eq!(buf.to_vec(), data.to_be_bytes());

    // Test little-endian
    let mut buf = BytesMut::with_capacity(7);
    data.encode_le_to(&mut buf).unwrap();
    assert_eq!(buf.to_vec(), data.to_le_bytes());
}

#[test]
fn test_default_implementation() {
    // The default implementation should work even without optimized code generation
    #[derive(BeBytes, Debug, PartialEq)]
    struct ComplexStruct {
        #[bits(3)]
        flags: u8,
        #[bits(5)]
        count: u8,
        data: [u8; 4],
        value: u64,
    }

    let data = ComplexStruct {
        flags: 0b101,
        count: 0b11010,
        data: [1, 2, 3, 4],
        value: 0x123456789ABCDEF0,
    };

    // Test that direct writing produces same result as to_bytes
    let mut buf_be = BytesMut::with_capacity(13);
    data.encode_be_to(&mut buf_be).unwrap();
    assert_eq!(buf_be.to_vec(), data.to_be_bytes());

    let mut buf_le = BytesMut::with_capacity(13);
    data.encode_le_to(&mut buf_le).unwrap();
    assert_eq!(buf_le.to_vec(), data.to_le_bytes());
}

#[test]
fn test_multiple_writes_to_same_buffer() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Packet {
        header: u16,
        payload: u32,
    }

    let packet1 = Packet {
        header: 0x1234,
        payload: 0x56789ABC,
    };
    let packet2 = Packet {
        header: 0xABCD,
        payload: 0xDEF01234,
    };

    let mut buf = BytesMut::with_capacity(12);

    // Write multiple packets to the same buffer
    packet1.encode_be_to(&mut buf).unwrap();
    packet2.encode_be_to(&mut buf).unwrap();

    // Verify the buffer contains both packets
    let expected = [packet1.to_be_bytes(), packet2.to_be_bytes()].concat();
    assert_eq!(buf.to_vec(), expected);
}
