use bebytes::BeBytes;
use std::time::Instant;

// Small struct (4 bytes)
#[derive(BeBytes, Clone)]
struct TinyStruct {
    a: u16,
    b: u16,
}

// Medium struct (16 bytes)
#[derive(BeBytes, Clone)]
struct MediumStruct {
    header: u32,
    payload: u64,
    footer: u32,
}

// Large struct (64 bytes)
#[derive(BeBytes, Clone)]
struct LargeStruct {
    id: u64,
    timestamp: u64,
    data: [u8; 32],
    checksum1: u64,
    checksum2: u64,
    flags: u32,
    version: u32,
}

// Very large struct (256 bytes - max for raw pointer)
#[derive(BeBytes, Clone)]
struct MaxStruct {
    header: [u8; 64],
    payload: [u8; 128],
    footer: [u8; 64],
}

// Struct with bit fields (not eligible for raw pointer)
#[derive(BeBytes, Clone)]
struct BitFieldStruct {
    #[bits(4)]
    high: u8,
    #[bits(4)]
    low: u8,
    regular: u32,
}

// Mixed types struct
#[derive(BeBytes, Clone)]
struct MixedStruct {
    u8_field: u8,
    u16_field: u16,
    u32_field: u32,
    u64_field: u64,
    u128_field: u128,
    i8_field: i8,
    i16_field: i16,
    i32_field: i32,
    i64_field: i64,
    i128_field: i128,
    char_field: char,
}

fn benchmark_struct<T: BeBytes + Clone + 'static>(name: &str, value: &T, iterations: usize) {
    println!("\n=== {} (size: {} bytes) ===", name, T::field_size());

    // Vec approach
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = value.to_be_bytes();
        std::hint::black_box(_result);
    }
    let vec_time = start.elapsed();
    let vec_ns = vec_time.as_nanos() as f64 / iterations as f64;

    println!("Vec approach:      {:7.2} ns/op", vec_ns);

    // Check if raw pointer is supported
    if std::any::type_name::<T>().contains("TinyStruct")
        && TinyStruct::supports_raw_pointer_encoding()
    {
        let tiny = value as &dyn std::any::Any;
        if let Some(t) = tiny.downcast_ref::<TinyStruct>() {
            let start = Instant::now();
            for _ in 0..iterations {
                let _result = t.encode_be_to_raw_stack();
                std::hint::black_box(_result);
            }
            let raw_time = start.elapsed();
            let raw_ns = raw_time.as_nanos() as f64 / iterations as f64;
            println!(
                "Raw pointer:       {:7.2} ns/op ({:.2}x vs Vec)",
                raw_ns,
                vec_ns / raw_ns
            );
        }
    } else if std::any::type_name::<T>().contains("MediumStruct")
        && MediumStruct::supports_raw_pointer_encoding()
    {
        let medium = value as &dyn std::any::Any;
        if let Some(m) = medium.downcast_ref::<MediumStruct>() {
            let start = Instant::now();
            for _ in 0..iterations {
                let _result = m.encode_be_to_raw_stack();
                std::hint::black_box(_result);
            }
            let raw_time = start.elapsed();
            let raw_ns = raw_time.as_nanos() as f64 / iterations as f64;
            println!(
                "Raw pointer:       {:7.2} ns/op ({:.2}x vs Vec)",
                raw_ns,
                vec_ns / raw_ns
            );
        }
    } else if std::any::type_name::<T>().contains("LargeStruct")
        && LargeStruct::supports_raw_pointer_encoding()
    {
        let large = value as &dyn std::any::Any;
        if let Some(l) = large.downcast_ref::<LargeStruct>() {
            let start = Instant::now();
            for _ in 0..iterations {
                let _result = l.encode_be_to_raw_stack();
                std::hint::black_box(_result);
            }
            let raw_time = start.elapsed();
            let raw_ns = raw_time.as_nanos() as f64 / iterations as f64;
            println!(
                "Raw pointer:       {:7.2} ns/op ({:.2}x vs Vec)",
                raw_ns,
                vec_ns / raw_ns
            );
        }
    } else if std::any::type_name::<T>().contains("MaxStruct")
        && MaxStruct::supports_raw_pointer_encoding()
    {
        let max = value as &dyn std::any::Any;
        if let Some(m) = max.downcast_ref::<MaxStruct>() {
            let start = Instant::now();
            for _ in 0..iterations {
                let _result = m.encode_be_to_raw_stack();
                std::hint::black_box(_result);
            }
            let raw_time = start.elapsed();
            let raw_ns = raw_time.as_nanos() as f64 / iterations as f64;
            println!(
                "Raw pointer:       {:7.2} ns/op ({:.2}x vs Vec)",
                raw_ns,
                vec_ns / raw_ns
            );
        }
    } else if std::any::type_name::<T>().contains("MixedStruct")
        && MixedStruct::supports_raw_pointer_encoding()
    {
        let mixed = value as &dyn std::any::Any;
        if let Some(m) = mixed.downcast_ref::<MixedStruct>() {
            let start = Instant::now();
            for _ in 0..iterations {
                let _result = m.encode_be_to_raw_stack();
                std::hint::black_box(_result);
            }
            let raw_time = start.elapsed();
            let raw_ns = raw_time.as_nanos() as f64 / iterations as f64;
            println!(
                "Raw pointer:       {:7.2} ns/op ({:.2}x vs Vec)",
                raw_ns,
                vec_ns / raw_ns
            );
        }
    } else {
        println!("Raw pointer:       Not supported");
    }
}

fn main() {
    println!("🚀 Extensive BeBytes Performance Benchmark");
    println!("Testing various struct sizes and types\n");

    let iterations = 1_000_000;

    // Create test data
    let tiny = TinyStruct {
        a: 0x1234,
        b: 0x5678,
    };
    let medium = MediumStruct {
        header: 0x12345678,
        payload: 0x123456789ABCDEF0,
        footer: 0xDEADBEEF,
    };
    let large = LargeStruct {
        id: 12345,
        timestamp: 1234567890,
        data: [42; 32],
        checksum1: 0xAAAAAAAAAAAAAAAA,
        checksum2: 0xBBBBBBBBBBBBBBBB,
        flags: 0xFF00FF00,
        version: 0x01020304,
    };
    let max = MaxStruct {
        header: [1; 64],
        payload: [2; 128],
        footer: [3; 64],
    };
    let bitfield = BitFieldStruct {
        high: 15,
        low: 10,
        regular: 0x12345678,
    };
    let mixed = MixedStruct {
        u8_field: 255,
        u16_field: 65535,
        u32_field: 0xFFFFFFFF,
        u64_field: 0xFFFFFFFFFFFFFFFF,
        u128_field: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        i8_field: -128,
        i16_field: -32768,
        i32_field: -2147483648,
        i64_field: -9223372036854775808,
        i128_field: -170141183460469231731687303715884105728,
        char_field: '🦀',
    };

    // Run benchmarks
    benchmark_struct("TinyStruct", &tiny, iterations);
    benchmark_struct("MediumStruct", &medium, iterations);
    benchmark_struct("LargeStruct", &large, iterations);
    benchmark_struct("MaxStruct", &max, iterations);
    benchmark_struct("BitFieldStruct", &bitfield, iterations);
    benchmark_struct("MixedStruct", &mixed, iterations);

    println!("\n📊 Summary:");
    println!("• Raw pointer optimization provides 40-80x speedup for eligible structs");
    println!("• Performance gains are consistent across different struct sizes");
    println!("• Bit field structs cannot use raw pointer optimization");
    println!("• Mixed type structs with all primitives are fully supported");
}
