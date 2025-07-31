use bebytes::BeBytes;

#[derive(BeBytes, Debug)]
struct SimpleStruct {
    a: u32,
    b: u16,
}

#[derive(BeBytes, Debug)]
struct BitFieldStruct {
    #[bits(4)]
    version: u8,
    #[bits(4)]
    header_len: u8,
    total_length: u16,
}

#[derive(BeBytes, Debug)]
struct VectorStruct {
    header: u32,
    #[With(size(64))]
    data: Vec<u8>,
}

fn main() {
    let simple = SimpleStruct {
        a: 0x12345678,
        b: 0xABCD,
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

    println!(
        "Simple struct optimal method: {}",
        SimpleStruct::optimal_serialization_method()
    );
    println!(
        "Bit field struct optimal method: {}",
        BitFieldStruct::optimal_serialization_method()
    );
    println!(
        "Vector struct optimal method: {}",
        VectorStruct::optimal_serialization_method()
    );

    // Test the optimal methods
    let simple_bytes = simple.to_be_bytes_optimal().unwrap();
    let bit_field_bytes = bit_field.to_be_bytes_optimal().unwrap();
    let vector_bytes = vector.to_be_bytes_optimal().unwrap();

    println!("Simple struct serialized {} bytes", simple_bytes.len());
    println!(
        "Bit field struct serialized {} bytes",
        bit_field_bytes.len()
    );
    println!("Vector struct serialized {} bytes", vector_bytes.len());

    // Test specific optimizations
    if SimpleStruct::supports_raw_pointer_encoding() {
        let raw_bytes = simple.encode_be_to_raw_stack();
        println!("Simple struct raw pointer: {:?}", raw_bytes);
    }

    // Test performance-aware serialization
    let simple_standard = simple.to_be_bytes();
    let simple_bytes_buf = simple.to_be_bytes_buf();
    let simple_optimal = simple.to_be_bytes_optimal().unwrap();

    println!("Standard: {} bytes", simple_standard.len());
    println!("Bytes buf: {} bytes", simple_bytes_buf.len());
    println!("Optimal: {} bytes", simple_optimal.len());
}
