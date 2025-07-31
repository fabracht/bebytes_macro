use bebytes::BeBytes;

// Test different struct types for optimization
#[derive(BeBytes, Debug, PartialEq)]
struct TinyStruct {
    value: u16,
}

#[derive(BeBytes, Debug, PartialEq)]
struct SmallStruct {
    a: u32,
    b: u16,
    c: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct MediumStruct {
    header: u64,
    payload: [u8; 32],
    checksum: u32,
}

#[derive(BeBytes, Debug, PartialEq)]
struct LargeStruct {
    id: u64,
    timestamp: u64,
    data: [u8; 200],
    footer: u32,
}

#[derive(BeBytes, Debug, PartialEq)]
struct BitFieldStruct {
    #[bits(4)]
    version: u8,
    #[bits(4)]
    header_len: u8,
    total_length: u16,
}

#[derive(BeBytes, Debug, PartialEq)]
struct VectorStruct {
    header: u32,
    #[With(size(64))]
    data: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct StringStruct {
    id: u32,
    #[With(size(16))]
    name: String,
}

#[derive(BeBytes, Debug, PartialEq)]
struct ComplexStruct {
    #[bits(4)]
    version: u8,
    #[bits(4)]
    flags: u8,
    length: u16,
    #[FromField(length)]
    payload: Vec<u8>,
}

#[test]
fn test_optimal_method_hints() {
    // Small structs should prefer raw pointer optimization
    assert_eq!(
        TinyStruct::optimal_serialization_method(),
        "encode_be_to_raw_stack() - 5.4x performance improvement"
    );
    assert_eq!(
        SmallStruct::optimal_serialization_method(),
        "encode_be_to_raw_stack() - 5.4x performance improvement"
    );
    assert_eq!(
        MediumStruct::optimal_serialization_method(),
        "encode_be_to_raw_stack() - 5.4x performance improvement"
    );

    // Large structs should use Bytes buffer
    assert_eq!(
        LargeStruct::optimal_serialization_method(),
        "to_be_bytes_buf() - 2.3x performance improvement"
    );

    // Bit field structs should use Bytes buffer (can't use raw pointer)
    assert_eq!(
        BitFieldStruct::optimal_serialization_method(),
        "to_be_bytes_buf() - 2.3x performance improvement"
    );

    // Complex structs should use standard approach
    assert_eq!(
        VectorStruct::optimal_serialization_method(),
        "to_be_bytes() - standard approach for complex types"
    );
    assert_eq!(
        StringStruct::optimal_serialization_method(),
        "to_be_bytes() - standard approach for complex types"
    );
    assert_eq!(
        ComplexStruct::optimal_serialization_method(),
        "to_be_bytes() - standard approach for complex types"
    );
}

#[test]
fn test_raw_pointer_support() {
    // Simple structs should support raw pointer encoding
    assert!(TinyStruct::supports_raw_pointer_encoding());
    assert!(SmallStruct::supports_raw_pointer_encoding());
    assert!(MediumStruct::supports_raw_pointer_encoding());

    // Large structs may or may not support raw pointer (depends on size limit)
    // This is implementation dependent

    // Bit field and complex structs should not support raw pointer
    assert!(!BitFieldStruct::supports_raw_pointer_encoding());
    assert!(!VectorStruct::supports_raw_pointer_encoding());
    assert!(!StringStruct::supports_raw_pointer_encoding());
    assert!(!ComplexStruct::supports_raw_pointer_encoding());
}

#[test]
fn test_optimal_serialization_methods() {
    let tiny = TinyStruct { value: 0x1234 };
    let small = SmallStruct {
        a: 0x12345678,
        b: 0xABCD,
        c: 0x42,
    };
    let bit_field = BitFieldStruct {
        version: 4,
        header_len: 5,
        total_length: 1024,
    };
    let vector = VectorStruct {
        header: 0x12345678,
        data: vec![0x42; 64],
    };

    // Test that optimal methods work
    let tiny_optimal = tiny.to_be_bytes_optimal().unwrap();
    let small_optimal = small.to_be_bytes_optimal().unwrap();
    let bit_field_optimal = bit_field.to_be_bytes_optimal().unwrap();
    let vector_optimal = vector.to_be_bytes_optimal().unwrap();

    // Verify the serialized data is correct by deserializing
    let tiny_standard = tiny.to_be_bytes();
    let small_standard = small.to_be_bytes();
    let bit_field_standard = bit_field.to_be_bytes();
    let vector_standard = vector.to_be_bytes();

    // The optimal methods should produce the same output as standard methods
    assert_eq!(tiny_optimal.len(), tiny_standard.len());
    assert_eq!(small_optimal.len(), small_standard.len());
    assert_eq!(bit_field_optimal.len(), bit_field_standard.len());
    assert_eq!(vector_optimal.len(), vector_standard.len());

    // Verify deserialization works
    let (tiny_decoded, _) = TinyStruct::try_from_be_bytes(&tiny_optimal).unwrap();
    let (small_decoded, _) = SmallStruct::try_from_be_bytes(&small_optimal).unwrap();
    let (bit_field_decoded, _) = BitFieldStruct::try_from_be_bytes(&bit_field_optimal).unwrap();
    let (vector_decoded, _) = VectorStruct::try_from_be_bytes(&vector_optimal).unwrap();

    assert_eq!(tiny_decoded, tiny);
    assert_eq!(small_decoded, small);
    assert_eq!(bit_field_decoded, bit_field);
    assert_eq!(vector_decoded, vector);
}

#[test]
fn test_raw_pointer_optimization() {
    let small = SmallStruct {
        a: 0x12345678,
        b: 0xABCD,
        c: 0x42,
    };

    if SmallStruct::supports_raw_pointer_encoding() {
        // Test big-endian raw pointer encoding
        let raw_be = small.encode_be_to_raw_stack();
        let standard_be = small.to_be_bytes();

        assert_eq!(raw_be.len(), standard_be.len());
        assert_eq!(raw_be.as_slice(), standard_be.as_slice());

        // Test little-endian raw pointer encoding
        let raw_le = small.encode_le_to_raw_stack();
        let standard_le = small.to_le_bytes();

        assert_eq!(raw_le.len(), standard_le.len());
        assert_eq!(raw_le.as_slice(), standard_le.as_slice());

        // Verify deserialization works with raw pointer data
        let (decoded_be, _) = SmallStruct::try_from_be_bytes(&raw_be).unwrap();
        let (decoded_le, _) = SmallStruct::try_from_le_bytes(&raw_le).unwrap();

        assert_eq!(decoded_be, small);
        assert_eq!(decoded_le, small);
    }
}

#[test]
fn test_bytes_buffer_optimization() {
    let bit_field = BitFieldStruct {
        version: 4,
        header_len: 5,
        total_length: 1024,
    };

    // Test that Bytes buffer methods work correctly
    let bytes_buf = bit_field.to_be_bytes_buf();
    let standard = bit_field.to_be_bytes();

    assert_eq!(bytes_buf.len(), standard.len());
    assert_eq!(bytes_buf.as_ref(), standard.as_slice());

    // Test optimal method uses Bytes buffer approach
    let optimal = bit_field.to_be_bytes_optimal().unwrap();
    assert_eq!(optimal.len(), standard.len());
    assert_eq!(optimal.as_ref(), standard.as_slice());
}

#[test]
fn test_complex_struct_fallback() {
    let vector = VectorStruct {
        header: 0x12345678,
        data: vec![0x42; 64],
    };

    // Complex structs should fall back to standard Vec approach
    let optimal = vector.to_be_bytes_optimal().unwrap();
    let standard = vector.to_be_bytes();

    assert_eq!(optimal.len(), standard.len());
    assert_eq!(optimal.as_ref(), standard.as_slice());

    // Verify deserialization
    let (decoded, _) = VectorStruct::try_from_be_bytes(&optimal).unwrap();
    assert_eq!(decoded, vector);
}

#[test]
fn test_little_endian_optimal_methods() {
    let small = SmallStruct {
        a: 0x12345678,
        b: 0xABCD,
        c: 0x42,
    };
    let bit_field = BitFieldStruct {
        version: 4,
        header_len: 5,
        total_length: 1024,
    };

    // Test little-endian optimal methods
    let small_optimal_le = small.to_le_bytes_optimal().unwrap();
    let bit_field_optimal_le = bit_field.to_le_bytes_optimal().unwrap();

    let small_standard_le = small.to_le_bytes();
    let bit_field_standard_le = bit_field.to_le_bytes();

    assert_eq!(small_optimal_le.len(), small_standard_le.len());
    assert_eq!(bit_field_optimal_le.len(), bit_field_standard_le.len());

    // Verify deserialization
    let (small_decoded, _) = SmallStruct::try_from_le_bytes(&small_optimal_le).unwrap();
    let (bit_field_decoded, _) = BitFieldStruct::try_from_le_bytes(&bit_field_optimal_le).unwrap();

    assert_eq!(small_decoded, small);
    assert_eq!(bit_field_decoded, bit_field);
}

#[test]
fn test_performance_consistency() {
    // Test that optimal methods are consistent across multiple calls
    let small = SmallStruct {
        a: 0x12345678,
        b: 0xABCD,
        c: 0x42,
    };

    let optimal1 = small.to_be_bytes_optimal().unwrap();
    let optimal2 = small.to_be_bytes_optimal().unwrap();
    let optimal3 = small.to_le_bytes_optimal().unwrap();

    assert_eq!(optimal1, optimal2);

    // BE and LE should be different (unless the struct is symmetric)
    // For this struct, they should be different due to multi-byte fields
    assert_ne!(optimal1, optimal3);
}

#[test]
fn test_buffer_reuse_helpers() {
    let small = SmallStruct {
        a: 0x12345678,
        b: 0xABCD,
        c: 0x42,
    };

    // Test batch buffer creation
    let mut batch_buffer = SmallStruct::create_batch_buffer_be(10);
    assert!(batch_buffer.capacity() >= SmallStruct::field_size() * 10);

    // Test reusable encoding
    small.encode_be_to_reused(&mut batch_buffer).unwrap();
    assert_eq!(batch_buffer.len(), SmallStruct::field_size());

    // Test multiple encodings to the same buffer
    small.encode_be_to_reused(&mut batch_buffer).unwrap();
    assert_eq!(batch_buffer.len(), SmallStruct::field_size() * 2);

    // Test little-endian batch buffer
    let mut le_buffer = SmallStruct::create_batch_buffer_le(5);
    small.encode_le_to_reused(&mut le_buffer).unwrap();
    assert_eq!(le_buffer.len(), SmallStruct::field_size());
}

#[test]
fn test_batch_serialization_performance() {
    let structs = vec![
        SmallStruct {
            a: 0x12345678,
            b: 0xABCD,
            c: 0x42,
        },
        SmallStruct {
            a: 0x87654321,
            b: 0xDCBA,
            c: 0x24,
        },
        SmallStruct {
            a: 0xABCDEF00,
            b: 0x1234,
            c: 0x56,
        },
    ];

    // Test batch serialization with reused buffer
    let mut batch_buffer = SmallStruct::create_batch_buffer_be(structs.len());

    for s in &structs {
        s.encode_be_to_reused(&mut batch_buffer).unwrap();
    }

    // Verify the buffer contains all serialized structs
    assert_eq!(
        batch_buffer.len(),
        SmallStruct::field_size() * structs.len()
    );

    // Test deserialization from the batch buffer
    let batch_data = batch_buffer.freeze();
    let mut offset = 0;

    for expected in &structs {
        let remaining = &batch_data[offset..];
        let (decoded, consumed) = SmallStruct::try_from_be_bytes(remaining).unwrap();
        assert_eq!(decoded, *expected);
        assert_eq!(consumed, SmallStruct::field_size());
        offset += consumed;
    }
}
