use bebytes::BeBytes;
use std::time::Instant;

#[derive(BeBytes, Clone)]
struct SimpleStruct {
    a: u8,
    b: u16,
    c: u32,
}

#[derive(BeBytes, Clone)]
struct ArrayStruct {
    header: u16,
    data: [u8; 8],
    footer: u32,
}

fn benchmark_raw_pointer_performance() {
    use bebytes::BytesMut;

    let simple = SimpleStruct {
        a: 42,
        b: 1337,
        c: 0xDEADBEEF,
    };
    let array = ArrayStruct {
        header: 0x1234,
        data: [1, 2, 3, 4, 5, 6, 7, 8],
        footer: 0xABCD,
    };

    let iterations = 1_000_000;
    println!("🚀 Raw Pointer Performance Benchmark");
    println!("Testing {} iterations per method\n", iterations);

    // === SimpleStruct Benchmark ===
    println!("=== SimpleStruct (u8 + u16 + u32) ===");

    // Vec approach
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = simple.to_be_bytes();
        std::hint::black_box(_result);
    }
    let vec_time = start.elapsed();
    let vec_ns = vec_time.as_nanos() as f64 / iterations as f64;

    // Direct BufMut approach
    let mut buf = BytesMut::with_capacity(SimpleStruct::field_size());
    let start = Instant::now();
    for _ in 0..iterations {
        buf.clear();
        simple.encode_be_to(&mut buf).unwrap();
        std::hint::black_box(&buf[..]);
    }
    let direct_time = start.elapsed();
    let direct_ns = direct_time.as_nanos() as f64 / iterations as f64;

    // Raw pointer approach
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = simple.encode_be_to_raw_stack();
        std::hint::black_box(_result);
    }
    let raw_time = start.elapsed();
    let raw_ns = raw_time.as_nanos() as f64 / iterations as f64;

    println!("Vec approach:      {:7.2} ns/op", vec_ns);
    println!(
        "Direct BufMut:     {:7.2} ns/op ({:.2}x vs Vec)",
        direct_ns,
        vec_ns / direct_ns
    );
    println!(
        "Raw pointer:       {:7.2} ns/op ({:.2}x vs Vec)",
        raw_ns,
        vec_ns / raw_ns
    );

    // === ArrayStruct Benchmark ===
    println!("\n=== ArrayStruct (u16 + [u8; 8] + u32) ===");

    // Vec approach
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = array.to_be_bytes();
        std::hint::black_box(_result);
    }
    let vec_time = start.elapsed();
    let vec_ns = vec_time.as_nanos() as f64 / iterations as f64;

    // Raw pointer approach
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = array.encode_be_to_raw_stack();
        std::hint::black_box(_result);
    }
    let raw_time = start.elapsed();
    let raw_ns = raw_time.as_nanos() as f64 / iterations as f64;

    println!("Vec approach:      {:7.2} ns/op", vec_ns);
    println!(
        "Raw pointer:       {:7.2} ns/op ({:.2}x vs Vec)",
        raw_ns,
        vec_ns / raw_ns
    );

    // Correctness verification
    println!("\n🔍 Correctness Verification:");

    let vec_result = simple.to_be_bytes();
    let raw_result = simple.encode_be_to_raw_stack();
    println!("SimpleStruct - Vec: {:?}", vec_result);
    println!("SimpleStruct - Raw: {:?}", raw_result);
    println!("Match: {}", vec_result == raw_result);

    let vec_result = array.to_be_bytes();
    let raw_result = array.encode_be_to_raw_stack();
    println!("ArrayStruct - Vec: {:?}", vec_result);
    println!("ArrayStruct - Raw: {:?}", raw_result);
    println!("Match: {}", vec_result == raw_result);

    println!("\n🎯 Performance Analysis:");
    println!("• Raw pointer approach eliminates ALL allocation overhead");
    println!("• Direct memory writes with compile-time known offsets");
    println!("• Zero abstraction cost - just pointer arithmetic and memcpy");
    println!("• Massive gains on simple structs with primitive fields");
}

fn main() {
    benchmark_raw_pointer_performance();
}
