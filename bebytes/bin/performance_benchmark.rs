#![allow(clippy::assign_op_pattern)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_possible_wrap)]

use bebytes::BeBytes;
use std::time::Instant;

fn main() {
    println!("=== BeBytes Performance Benchmark Suite ===");
    println!("Use profiling tools (perf, valgrind, etc.) to analyze performance");

    // Run basic verification first
    println!("\nRunning basic verification tests...");
    run_verification_tests();

    println!("\n=== STARTING PERFORMANCE BENCHMARKS ===\n");

    // Run each benchmark scenario
    benchmark_primitive_serialization();
    benchmark_bit_field_operations();
    benchmark_enum_operations();
    benchmark_vector_operations();
    benchmark_nested_structures();
    benchmark_array_operations();
    benchmark_mixed_scenarios();

    println!("\n=== BENCHMARKS COMPLETED ===");
    println!("Profile data should now be available for analysis");
}

// ============ BENCHMARK FUNCTIONS ============

fn benchmark_primitive_serialization() {
    println!("=== PRIMITIVE SERIALIZATION BENCHMARK ===");

    const ITERATIONS: usize = 1_000_000;
    let mut total_bytes = 0;

    // Benchmark U8 serialization
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let val = U8 {
            first: (i % 2) as u8,
            second: (i % 8) as u8,
            third: (i % 16) as u8,
            fourth: (i % 256) as u8,
        };
        let bytes = val.to_be_bytes();
        total_bytes += bytes.len();
    }
    let duration = start.elapsed();
    println!(
        "U8 serialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark U8 deserialization
    let test_bytes = U8 {
        first: 1,
        second: 2,
        third: 3,
        fourth: 4,
    }
    .to_be_bytes();
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let (decoded, _) = U8::try_from_be_bytes(&test_bytes).unwrap();
        // Use the decoded value to prevent optimization
        std::hint::black_box(decoded);
    }
    let duration = start.elapsed();
    println!(
        "U8 deserialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark U16 serialization
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let val = U16 {
            first: (i % 2) as u8,
            second: (i % 16384) as u16,
            fourth: (i % 2) as u8,
        };
        let bytes = val.to_be_bytes();
        total_bytes += bytes.len();
    }
    let duration = start.elapsed();
    println!(
        "U16 serialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark U32 serialization
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let val = U32 {
            first: (i % 2) as u8,
            second: (i % 1_073_741_824) as u32,
            fourth: (i % 2) as u8,
        };
        let bytes = val.to_be_bytes();
        total_bytes += bytes.len();
    }
    let duration = start.elapsed();
    println!(
        "U32 serialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    println!("Total bytes processed: {total_bytes}");
    println!();
}

fn benchmark_bit_field_operations() {
    println!("=== BIT FIELD OPERATIONS BENCHMARK ===");

    const ITERATIONS: usize = 500_000;
    let mut total_bytes = 0;

    // Benchmark bit field serialization
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let val = ErrorEstimateMini {
            s_bit: (i % 2) as u8,
            z_bit: (i % 2) as u8,
            scale: (i % 64) as u8,
            multiplier: (i % 1000) as u32,
        };
        let bytes = val.to_be_bytes();
        total_bytes += bytes.len();
    }
    let duration = start.elapsed();
    println!(
        "Complex bit field serialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark bit field deserialization
    let test_bytes = ErrorEstimateMini {
        s_bit: 1,
        z_bit: 0,
        scale: 63,
        multiplier: 1000,
    }
    .to_be_bytes();
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let (decoded, _) = ErrorEstimateMini::try_from_be_bytes(&test_bytes).unwrap();
        std::hint::black_box(decoded);
    }
    let duration = start.elapsed();
    println!(
        "Complex bit field deserialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark cross-byte bit operations
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let val = U16 {
            first: (i % 2) as u8,
            second: (i % 16384) as u16,
            fourth: (i % 2) as u8,
        };
        let bytes = val.to_be_bytes();
        let (decoded, _) = U16::try_from_be_bytes(&bytes).unwrap();
        std::hint::black_box(decoded);
    }
    let duration = start.elapsed();
    println!(
        "Cross-byte bit operations: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    println!("Total bytes processed: {total_bytes}");
    println!();
}

fn benchmark_enum_operations() {
    println!("=== ENUM OPERATIONS BENCHMARK ===");

    const ITERATIONS: usize = 1_000_000;
    let mut total_bytes = 0;

    // Benchmark regular enum serialization
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let val = match i % 3 {
            0 => DummyEnum::SetupResponse,
            1 => DummyEnum::ServerStart,
            _ => DummyEnum::SetupRequest,
        };
        let bytes = val.to_be_bytes();
        total_bytes += bytes.len();
    }
    let duration = start.elapsed();
    println!(
        "Regular enum serialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark auto-sized enum operations
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let val = PacketHeader {
            version: (i % 16) as u8,
            status: (i % 4) as u8,   // 0=Idle, 1=Running, 2=Paused, 3=Stopped
            priority: (i % 3) as u8, // 0=Low, 1=Medium, 2=High
        };
        let bytes = val.to_be_bytes();
        total_bytes += bytes.len();
    }
    let duration = start.elapsed();
    println!(
        "Bit-packed struct serialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark flag enum operations
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let val = SecurityContext {
            user_id: (i % 32) as u8,
            group_id: (i % 8) as u8,
            permissions: (FilePermissions::Read | FilePermissions::Write),
            network_flags: (NetworkFlags::Connected | NetworkFlags::Authenticated),
        };
        let bytes = val.to_be_bytes();
        total_bytes += bytes.len();
    }
    let duration = start.elapsed();
    println!(
        "Flag enum serialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    println!("Total bytes processed: {total_bytes}");
    println!();
}

fn benchmark_vector_operations() {
    println!("=== VECTOR OPERATIONS BENCHMARK ===");

    const ITERATIONS: usize = 100_000;
    let mut total_bytes = 0;

    // Benchmark fixed-size vector serialization
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let val = WithSizeStruct {
            innocent: (i % 256) as u8,
            real_tail: vec![
                (i % 256) as u8,
                ((i + 1) % 256) as u8,
                ((i + 2) % 256) as u8,
            ],
        };
        let bytes = val.to_be_bytes();
        total_bytes += bytes.len();
    }
    let duration = start.elapsed();
    println!(
        "Fixed-size vector serialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark dynamic vector serialization
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let size = (i % 10) as u8;
        let val = WithTailingVec {
            pre_tail: size,
            tail: (0..size).map(|j| (j + (i % 256) as u8)).collect(),
            post_tail: 99,
        };
        let bytes = val.to_be_bytes();
        total_bytes += bytes.len();
    }
    let duration = start.elapsed();
    println!(
        "Dynamic vector serialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark vector deserialization
    let test_bytes = WithSizeStruct {
        innocent: 1,
        real_tail: vec![1, 2, 3],
    }
    .to_be_bytes();
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let (decoded, _) = WithSizeStruct::try_from_be_bytes(&test_bytes).unwrap();
        std::hint::black_box(decoded);
    }
    let duration = start.elapsed();
    println!(
        "Vector deserialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    println!("Total bytes processed: {total_bytes}");
    println!();
}

fn benchmark_nested_structures() {
    println!("=== NESTED STRUCTURES BENCHMARK ===");

    const ITERATIONS: usize = 50_000;
    let mut total_bytes = 0;

    // Benchmark nested struct serialization
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let dummy = DummyStruct {
            dummy0: [(i % 256) as u8, ((i + 1) % 256) as u8],
            dummy1: (i % 2) as u8,
            dummy2: (i % 128) as u8,
        };

        let error_estimate = ErrorEstimate {
            s_bit: (i % 2) as u8,
            z_bit: (i % 2) as u8,
            scale: (i % 64) as u8,
            dummy_struct: dummy.clone(),
        };

        let nested = NestedStruct {
            dummy_struct: dummy,
            optional_number: if i % 2 == 0 { Some(i as i32) } else { None },
            error_estimate,
        };

        let bytes = nested.to_be_bytes();
        total_bytes += bytes.len();
    }
    let duration = start.elapsed();
    println!(
        "Nested struct serialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark nested struct deserialization
    let test_nested = NestedStruct {
        dummy_struct: DummyStruct {
            dummy0: [1, 2],
            dummy1: 1,
            dummy2: 2,
        },
        optional_number: Some(42),
        error_estimate: ErrorEstimate {
            s_bit: 1,
            z_bit: 0,
            scale: 63,
            dummy_struct: DummyStruct {
                dummy0: [3, 4],
                dummy1: 0,
                dummy2: 1,
            },
        },
    };
    let test_bytes = test_nested.to_be_bytes();

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let (decoded, _) = NestedStruct::try_from_be_bytes(&test_bytes).unwrap();
        std::hint::black_box(decoded);
    }
    let duration = start.elapsed();
    println!(
        "Nested struct deserialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    println!("Total bytes processed: {total_bytes}");
    println!();
}

fn benchmark_array_operations() {
    println!("=== ARRAY OPERATIONS BENCHMARK ===");

    const ITERATIONS: usize = 200_000;
    let mut total_bytes = 0;

    // Benchmark array serialization
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let val = ArrayedStruct {
            mode: Modes {
                bits: (i % 256) as u8,
            },
            key_id: [(i % 256) as u8],
            token: [(i % 256) as u8, ((i + 1) % 256) as u8],
            client_iv: [
                (i % 256) as u8,
                ((i + 1) % 256) as u8,
                ((i + 2) % 256) as u8,
            ],
        };
        let bytes = val.to_be_bytes();
        total_bytes += bytes.len();
    }
    let duration = start.elapsed();
    println!(
        "Array serialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark array deserialization
    let test_bytes = ArrayedStruct {
        mode: Modes { bits: 1 },
        key_id: [2],
        token: [3, 4],
        client_iv: [5, 6, 7],
    }
    .to_be_bytes();

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let (decoded, _) = ArrayedStruct::try_from_be_bytes(&test_bytes).unwrap();
        std::hint::black_box(decoded);
    }
    let duration = start.elapsed();
    println!(
        "Array deserialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    println!("Total bytes processed: {total_bytes}");
    println!();
}

fn benchmark_mixed_scenarios() {
    println!("=== MIXED SCENARIOS BENCHMARK ===");

    const ITERATIONS: usize = 10_000;
    let mut total_bytes = 0;

    // Benchmark complex mixed struct serialization
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let complete = CompleteFunctionality {
            first: (i % 2) as u8,
            second: (i % 8) as u8,
            third: (i % 16) as u8,
            with_size: vec![
                (i % 256) as u8,
                ((i + 1) % 256) as u8,
                ((i + 2) % 256) as u8,
            ],
            fourth: (i % 256) as u8,
            body: (0..(i % 10)).map(|j| ((i + j) % 256) as u8).collect(),
            u_16: U16 {
                first: (i % 2) as u8,
                second: (i % 16384) as u16,
                fourth: (i % 2) as u8,
            },
            arrayed: ArrayedStruct {
                mode: Modes {
                    bits: (i % 256) as u8,
                },
                key_id: [(i % 256) as u8],
                token: [(i % 256) as u8, ((i + 1) % 256) as u8],
                client_iv: [
                    (i % 256) as u8,
                    ((i + 1) % 256) as u8,
                    ((i + 2) % 256) as u8,
                ],
            },
            dummy_enum: match i % 3 {
                0 => DummyEnum::SetupResponse,
                1 => DummyEnum::ServerStart,
                _ => DummyEnum::SetupRequest,
            },
            optional: if i % 2 == 0 { Some(i as i32) } else { None },
            modes: Modes {
                bits: (i % 256) as u8,
            },
            vecty: WithTailingVec {
                pre_tail: (i % 5) as u8,
                tail: (0..(i % 5)).map(|j| ((i + j) % 256) as u8).collect(),
                post_tail: 99,
            },
            u_32: U32 {
                first: (i % 2) as u8,
                second: (i % 1_073_741_824) as u32,
                fourth: (i % 2) as u8,
            },
            rattle: (0..(i % 3)).map(|j| ((i + j) % 256) as u8).collect(),
        };

        let bytes = complete.to_be_bytes();
        total_bytes += bytes.len();
    }
    let duration = start.elapsed();
    println!(
        "Complex mixed serialization: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Memory allocation stress test
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let size = (i % 100) as u8;
        let dns_name = DnsName {
            segments: (0..3)
                .map(|j| DnsNameSegment {
                    length: size,
                    segment: (0..size)
                        .map(|k| ((i + j + k as usize) % 256) as u8)
                        .collect(),
                })
                .collect(),
        };
        let bytes = dns_name.to_be_bytes();
        total_bytes += bytes.len();
    }
    let duration = start.elapsed();
    println!(
        "Memory allocation stress test: {} iterations in {:?} ({:.2} ns/op)",
        ITERATIONS,
        duration,
        duration.as_nanos() as f64 / ITERATIONS as f64
    );

    println!("Total bytes processed: {total_bytes}");
    println!();
}

// ============ VERIFICATION FUNCTIONS ============

fn run_verification_tests() {
    // Basic roundtrip test
    let original = U8 {
        first: 1,
        second: 2,
        third: 3,
        fourth: 4,
    };
    let bytes = original.to_be_bytes();
    let (decoded, _) = U8::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(original, decoded);

    // Enum test
    let original = DummyEnum::ServerStart;
    let bytes = original.to_be_bytes();
    let (decoded, _) = DummyEnum::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(original, decoded);

    // Vector test
    let original = WithSizeStruct {
        innocent: 1,
        real_tail: vec![1, 2, 3],
    };
    let bytes = original.to_be_bytes();
    let (decoded, _) = WithSizeStruct::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(original, decoded);

    println!("✓ All verification tests passed");
}

// ============ STRUCTURE DEFINITIONS ============

#[derive(BeBytes, Debug, PartialEq)]
struct U8 {
    #[bits(1)]
    first: u8,
    #[bits(3)]
    second: u8,
    #[bits(4)]
    third: u8,
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
struct U16 {
    #[bits(1)]
    first: u8,
    #[bits(14)]
    second: u16,
    #[bits(1)]
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct U32 {
    #[bits(1)]
    first: u8,
    #[bits(30)]
    second: u32,
    #[bits(1)]
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
pub enum DummyEnum {
    SetupResponse,
    ServerStart,
    SetupRequest,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
pub struct DummyStruct {
    pub dummy0: [u8; 2],
    #[bits(1)]
    pub dummy1: u8,
    #[bits(7)]
    pub dummy2: u8,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
pub struct ErrorEstimate {
    #[bits(1)]
    pub s_bit: u8,
    #[bits(1)]
    pub z_bit: u8,
    #[bits(6)]
    pub scale: u8,
    pub dummy_struct: DummyStruct,
}

#[derive(BeBytes, Debug, PartialEq)]
pub struct ErrorEstimateMini {
    #[bits(1)]
    pub s_bit: u8,
    #[bits(1)]
    pub z_bit: u8,
    #[bits(6)]
    pub scale: u8,
    pub multiplier: u32,
}

#[derive(BeBytes, Debug, PartialEq)]
pub struct NestedStruct {
    pub dummy_struct: DummyStruct,
    pub optional_number: Option<i32>,
    pub error_estimate: ErrorEstimate,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
pub struct ArrayedStruct {
    pub mode: Modes,
    pub key_id: [u8; 1],
    pub token: [u8; 2],
    pub client_iv: [u8; 3],
}

#[derive(BeBytes, Debug, PartialEq, Clone, Default)]
pub struct Modes {
    pub bits: u8,
}

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct WithTailingVec {
    pre_tail: u8,
    #[FromField(pre_tail)]
    tail: Vec<u8>,
    post_tail: u8,
}

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct WithSizeStruct {
    innocent: u8,
    #[With(size(3))]
    real_tail: Vec<u8>,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct DnsNameSegment {
    length: u8,
    #[FromField(length)]
    segment: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct DnsName {
    segments: Vec<DnsNameSegment>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct CompleteFunctionality {
    #[bits(1)]
    first: u8,
    #[bits(3)]
    second: u8,
    #[bits(4)]
    third: u8,
    #[With(size(3))]
    with_size: Vec<u8>,
    fourth: u8,
    #[FromField(fourth)]
    body: Vec<u8>,
    u_16: U16,
    arrayed: ArrayedStruct,
    dummy_enum: DummyEnum,
    optional: Option<i32>,
    modes: Modes,
    vecty: WithTailingVec,
    u_32: U32,
    rattle: Vec<u8>,
}

// ============ Enum Bit Packing Examples ============

#[derive(BeBytes, Debug, PartialEq)]
struct PacketHeader {
    #[bits(4)]
    version: u8,
    #[bits(2)]
    status: u8,
    #[bits(2)]
    priority: u8,
}

// ============ Flag Enum Examples ============

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[bebytes(flags)]
enum FilePermissions {
    None = 0,
    Read = 1,
    Write = 2,
    Execute = 4,
    Delete = 8,
}

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[bebytes(flags)]
enum NetworkFlags {
    Connected = 1,
    Authenticated = 2,
    Encrypted = 4,
    Compressed = 8,
    KeepAlive = 16,
}

#[derive(BeBytes, Debug, PartialEq)]
struct SecurityContext {
    #[bits(5)]
    user_id: u8,
    #[bits(3)]
    group_id: u8,
    permissions: u8,   // Store FilePermissions flags
    network_flags: u8, // Store NetworkFlags
}
