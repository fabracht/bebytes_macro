use bebytes::{BeBytes, BytesMut};
use std::time::Instant;

#[derive(BeBytes, Clone)]
struct NetworkPacket {
    #[bits(4)]
    version: u8,
    #[bits(4)]
    flags: u8,
    source: u32,
    dest: u32,
    payload_size: u16,
}

#[derive(BeBytes, Clone)]
struct MediumStruct {
    header: u64,
    data: [u8; 16],
    checksum: u32,
    footer: u16,
}

fn benchmark_bytes_performance() {
    let iterations = 1_000_000;

    let packet = NetworkPacket {
        version: 4,
        flags: 0b1010,
        source: 0x12345678,
        dest: 0x87654321,
        payload_size: 1024,
    };

    let medium = MediumStruct {
        header: 0xDEADBEEFCAFEBABE,
        data: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        checksum: 0xABCDEF12,
        footer: 0x1234,
    };

    println!("🚀 bytes Crate Performance Benchmark");
    println!("Testing {} iterations per method\n", iterations);

    // === NetworkPacket (with bit fields) ===
    println!("=== NetworkPacket (bit fields + primitives) ===");

    // Vec approach
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = packet.to_be_bytes();
        std::hint::black_box(_result);
    }
    let vec_time = start.elapsed();
    let vec_ns = vec_time.as_nanos() as f64 / iterations as f64;

    // Bytes buffer approach
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = packet.to_be_bytes_buf();
        std::hint::black_box(_result);
    }
    let bytes_time = start.elapsed();
    let bytes_ns = bytes_time.as_nanos() as f64 / iterations as f64;

    // Direct BufMut approach (structs with bit fields use fallback)
    let mut buf = BytesMut::with_capacity(NetworkPacket::field_size());
    let start = Instant::now();
    for _ in 0..iterations {
        buf.clear();
        packet.encode_be_to(&mut buf).unwrap();
        std::hint::black_box(&buf[..]);
    }
    let direct_time = start.elapsed();
    let direct_ns = direct_time.as_nanos() as f64 / iterations as f64;

    println!("Vec approach:      {:7.2} ns/op", vec_ns);
    println!(
        "Bytes buffer:      {:7.2} ns/op ({:.2}x vs Vec)",
        bytes_ns,
        vec_ns / bytes_ns
    );
    println!(
        "Direct BufMut:     {:7.2} ns/op ({:.2}x vs Vec)",
        direct_ns,
        vec_ns / direct_ns
    );

    // === MediumStruct (no bit fields - eligible for optimizations) ===
    println!("\n=== MediumStruct (no bit fields, primitives only) ===");

    // Vec approach
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = medium.to_be_bytes();
        std::hint::black_box(_result);
    }
    let vec_time = start.elapsed();
    let vec_ns = vec_time.as_nanos() as f64 / iterations as f64;

    // Bytes buffer approach
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = medium.to_be_bytes_buf();
        std::hint::black_box(_result);
    }
    let bytes_time = start.elapsed();
    let bytes_ns = bytes_time.as_nanos() as f64 / iterations as f64;

    // Direct BufMut approach (optimized for non-bit-field structs)
    let mut buf = BytesMut::with_capacity(MediumStruct::field_size());
    let start = Instant::now();
    for _ in 0..iterations {
        buf.clear();
        medium.encode_be_to(&mut buf).unwrap();
        std::hint::black_box(&buf[..]);
    }
    let direct_time = start.elapsed();
    let direct_ns = direct_time.as_nanos() as f64 / iterations as f64;

    // Raw pointer approach (if available)
    if MediumStruct::supports_raw_pointer_encoding() {
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = medium.encode_be_to_raw_stack();
            std::hint::black_box(_result);
        }
        let raw_time = start.elapsed();
        let raw_ns = raw_time.as_nanos() as f64 / iterations as f64;

        println!("Vec approach:      {:7.2} ns/op", vec_ns);
        println!(
            "Bytes buffer:      {:7.2} ns/op ({:.2}x vs Vec)",
            bytes_ns,
            vec_ns / bytes_ns
        );
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
    } else {
        println!("Vec approach:      {:7.2} ns/op", vec_ns);
        println!(
            "Bytes buffer:      {:7.2} ns/op ({:.2}x vs Vec)",
            bytes_ns,
            vec_ns / bytes_ns
        );
        println!(
            "Direct BufMut:     {:7.2} ns/op ({:.2}x vs Vec)",
            direct_ns,
            vec_ns / direct_ns
        );
        println!("Raw pointer:       Not available (struct has bit fields or other limitations)");
    }

    // Correctness verification
    println!("\n🔍 Correctness Verification:");

    let packet_vec = packet.to_be_bytes();
    let packet_bytes = packet.to_be_bytes_buf();
    println!(
        "NetworkPacket - Vec vs Bytes match: {}",
        packet_vec == packet_bytes.as_ref()
    );

    let medium_vec = medium.to_be_bytes();
    let medium_bytes = medium.to_be_bytes_buf();
    println!(
        "MediumStruct - Vec vs Bytes match: {}",
        medium_vec == medium_bytes.as_ref()
    );

    println!("\n🎯 Buffer Management Benefits:");
    println!("• Internal buffer types with no external dependencies");
    println!("• BytesMut for efficient writing, Bytes for immutable results");
    println!("• Direct BufMut writing to existing buffers");
    println!("• Compatible with existing Vec<u8> APIs for backward compatibility");
}

fn main() {
    benchmark_bytes_performance();
}
