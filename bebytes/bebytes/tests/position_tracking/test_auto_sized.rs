use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
enum SmallEnum {
    A = 0,
    B = 1,
    C = 2,
}

#[derive(BeBytes, Debug, PartialEq)]
struct TestPositionTracking {
    #[bits(3)]
    before_bits: u8,
    
    #[bits()]  // Auto-sized, should be 2 bits
    auto_enum: SmallEnum,
    
    #[bits(3)]
    after_bits: u8,  // This field's position depends on auto_enum's size
    
    regular_field: u8,  // This should be byte-aligned
}

#[test]
fn test_position_tracking() {
    let data = TestPositionTracking {
        before_bits: 0x7,  // 3 bits: 111
        auto_enum: SmallEnum::C,  // 2 bits: 10
        after_bits: 0x5,  // 3 bits: 101
        regular_field: 0xFF,
    };
    
    let bytes = data.to_be_bytes();
    println\!("Serialized bytes: {:?}", bytes);
    
    // Expected layout:
    // Byte 0: [111][10][101] = 11110101 = 0xF5
    // Byte 1: 0xFF
    assert_eq\!(bytes.len(), 2);
    assert_eq\!(bytes[0], 0xF5);
    assert_eq\!(bytes[1], 0xFF);
    
    let (decoded, consumed) = TestPositionTracking::try_from_be_bytes(&bytes).unwrap();
    assert_eq\!(consumed, 2);
    assert_eq\!(decoded, data);
}

#[test] 
fn test_compile_time_optimizations() {
    // This test checks if compile-time optimizations work correctly
    // when we know positions at compile time
    
    #[derive(BeBytes, Debug, PartialEq)]
    struct NoAutoSized {
        #[bits(3)]
        field1: u8,
        #[bits(5)]
        field2: u8,
        field3: u16,  // Should be byte-aligned
    }
    
    #[derive(BeBytes, Debug, PartialEq)]
    struct WithAutoSized {
        #[bits(3)]
        field1: u8,
        #[bits()]
        auto_field: SmallEnum,  // 2 bits
        #[bits(3)]
        field2: u8,
        field3: u16,  // Position unknown at compile time?
    }
    
    // Check if both generate similar code
    let no_auto = NoAutoSized {
        field1: 0x7,
        field2: 0x1F,
        field3: 0x1234,
    };
    
    let with_auto = WithAutoSized {
        field1: 0x7,
        auto_field: SmallEnum::B,
        field2: 0x5,
        field3: 0x1234,
    };
    
    let bytes1 = no_auto.to_be_bytes();
    let bytes2 = with_auto.to_be_bytes();
    
    println\!("NoAutoSized bytes: {:?}", bytes1);
    println\!("WithAutoSized bytes: {:?}", bytes2);
    
    assert_eq\!(bytes1.len(), 3);
    assert_eq\!(bytes2.len(), 3);
}
EOF < /dev/null