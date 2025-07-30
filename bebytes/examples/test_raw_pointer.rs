use bebytes::BeBytes;

#[derive(BeBytes)]
struct SimpleStruct {
    a: u8,
    b: u16,
    c: u32,
}

fn main() {
    let s = SimpleStruct {
        a: 42,
        b: 1337,
        c: 0xDEADBEEF,
    };

    // Test if raw pointer methods are available
    println!("Struct size: {}", SimpleStruct::field_size());
    println!(
        "Supports raw pointer: {}",
        SimpleStruct::supports_raw_pointer_encoding()
    );

    // Test raw pointer stack method
    let stack_result = s.encode_be_to_raw_stack();
    println!("Raw stack result: {:?}", stack_result);

    // Compare with regular method
    let vec_result = s.to_be_bytes();
    println!("Vec result: {:?}", vec_result);
    println!(
        "Results match: {}",
        stack_result.as_slice() == vec_result.as_slice()
    );
}
