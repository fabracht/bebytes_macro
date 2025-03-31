use bebytes::BeBytes;

// This struct contains a bounded vector using FromField
#[derive(BeBytes, Debug, PartialEq, Clone)]
struct WithBoundedVector {
    size: u8,
    #[FromField(size)]
    bounded_vec: Vec<u8>, // Size constrained by the 'size' field
}

// This should compile successfully since WithBoundedVector has a bounded vector
#[derive(BeBytes, Debug, PartialEq, Clone)]
struct SafeNestedVector {
    first_field: u8,
    nested_field: WithBoundedVector, // This is safe because it uses a bounded vector
    last_field: u32,
}

// Another safe approach using the With attribute
#[derive(BeBytes, Debug, PartialEq, Clone)]
struct WithFixedSizeVector {
    #[With(size(4))]
    fixed_vec: Vec<u8>, // Fixed size of 4 bytes
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct AnotherSafeNested {
    some_field: u16,
    another_nested: WithFixedSizeVector,
    trailing_data: u64,
}

fn main() {
    // Test serialization/deserialization of safe nested struct
    let safe = SafeNestedVector {
        first_field: 1,
        nested_field: WithBoundedVector {
            size: 3,
            bounded_vec: vec![2, 3, 4],
        },
        last_field: 5,
    };
    
    let bytes = safe.to_be_bytes();
    let (deserialized, _) = SafeNestedVector::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(safe, deserialized);
    
    // Test fixed size vector nested struct
    let another = AnotherSafeNested {
        some_field: 10,
        another_nested: WithFixedSizeVector {
            fixed_vec: vec![1, 2, 3, 4],
        },
        trailing_data: 42,
    };
    
    let bytes = another.to_be_bytes();
    let (deserialized, _) = AnotherSafeNested::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(another, deserialized);
}