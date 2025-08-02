use bebytes::BeBytes;
use bytes::BytesMut;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use std::hint::black_box;

// ============ Test Structures ============

#[derive(BeBytes, Clone, Debug)]
struct TinyStruct {
    a: u16,
    b: u16,
}

#[derive(BeBytes, Clone, Debug)]
struct SmallStruct {
    header: u32,
    flags: u16,
    data: u16,
}

#[derive(BeBytes, Clone, Debug)]
struct MediumStruct {
    header: u64,
    payload: [u8; 32],
    checksum: u32,
    footer: u16,
}

#[derive(BeBytes, Clone, Debug)]
struct LargeStruct {
    id: u64,
    timestamp: u64,
    data: [u8; 128],
    metadata: [u8; 64],
    checksum1: u64,
    checksum2: u64,
    flags: u32,
    version: u32,
}

#[derive(BeBytes, Clone, Debug)]
struct BitFieldStruct {
    #[bits(4)]
    version: u8,
    #[bits(4)]
    flags: u8,
    #[bits(12)]
    length: u16,
    #[bits(4)]
    type_field: u8,
    payload: u32,
}

#[derive(BeBytes, Clone, Debug)]
struct VectorStruct {
    header: u32,
    #[With(size(16))]
    fixed_data: Vec<u8>,
    footer: u16,
}

#[derive(BeBytes, Clone, Debug)]
struct DynamicVectorStruct {
    length: u8,
    #[FromField(length)]
    data: Vec<u8>,
    checksum: u32,
}

#[derive(BeBytes, Clone, Debug)]
struct MixedStruct {
    #[bits(2)]
    version: u8,
    #[bits(6)]
    flags: u8,
    length: u16,
    #[FromField(length)]
    payload: Vec<u8>,
    footer: u32,
}

// ============ Benchmark Helper Functions ============

fn create_tiny_struct() -> TinyStruct {
    TinyStruct {
        a: 0x1234,
        b: 0x5678,
    }
}

fn create_small_struct() -> SmallStruct {
    SmallStruct {
        header: 0x12345678,
        flags: 0xABCD,
        data: 0xEF01,
    }
}

fn create_medium_struct() -> MediumStruct {
    MediumStruct {
        header: 0x123456789ABCDEF0,
        payload: [0x42; 32],
        checksum: 0xDEADBEEF,
        footer: 0x1234,
    }
}

fn create_large_struct() -> LargeStruct {
    LargeStruct {
        id: 0x123456789ABCDEF0,
        timestamp: 1234567890,
        data: [0xAA; 128],
        metadata: [0xBB; 64],
        checksum1: 0xDEADBEEFCAFEBABE,
        checksum2: 0x0123456789ABCDEF,
        flags: 0xFF00FF00,
        version: 0x01020304,
    }
}

fn create_bit_field_struct() -> BitFieldStruct {
    BitFieldStruct {
        version: 4,
        flags: 10,
        length: 1024,
        type_field: 7,
        payload: 0x12345678,
    }
}

fn create_vector_struct() -> VectorStruct {
    VectorStruct {
        header: 0x12345678,
        fixed_data: vec![0x42; 16],
        footer: 0xABCD,
    }
}

fn create_dynamic_vector_struct(size: u8) -> DynamicVectorStruct {
    DynamicVectorStruct {
        length: size,
        data: vec![0x42; size as usize],
        checksum: 0xDEADBEEF,
    }
}

fn create_mixed_struct(payload_size: u16) -> MixedStruct {
    MixedStruct {
        version: 2,
        flags: 63,
        length: payload_size,
        payload: vec![0x42; payload_size as usize],
        footer: 0xDEADBEEF,
    }
}

// ============ Serialization Benchmarks ============

fn bench_serialization_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization_methods");

    // Tiny struct benchmarks
    let tiny = create_tiny_struct();
    group.throughput(Throughput::Bytes(TinyStruct::field_size() as u64));

    group.bench_function("tiny_to_be_bytes", |b| {
        b.iter(|| black_box(tiny.to_be_bytes()))
    });

    group.bench_function("tiny_to_be_bytes_buf", |b| {
        b.iter(|| black_box(tiny.to_be_bytes_buf()))
    });

    group.bench_function("tiny_encode_be_to", |b| {
        b.iter_batched(
            || BytesMut::with_capacity(TinyStruct::field_size()),
            |mut buf| {
                black_box(tiny.encode_be_to(&mut buf).unwrap());
                black_box(buf);
            },
            BatchSize::SmallInput,
        )
    });

    // Medium struct benchmarks
    let medium = create_medium_struct();
    group.throughput(Throughput::Bytes(MediumStruct::field_size() as u64));

    group.bench_function("medium_to_be_bytes", |b| {
        b.iter(|| black_box(medium.to_be_bytes()))
    });

    group.bench_function("medium_to_be_bytes_buf", |b| {
        b.iter(|| black_box(medium.to_be_bytes_buf()))
    });

    group.bench_function("medium_encode_be_to", |b| {
        b.iter_batched(
            || BytesMut::with_capacity(MediumStruct::field_size()),
            |mut buf| {
                black_box(medium.encode_be_to(&mut buf).unwrap());
                black_box(buf);
            },
            BatchSize::SmallInput,
        )
    });

    // Test raw pointer if available
    if MediumStruct::supports_raw_pointer_encoding() {
        group.bench_function("medium_raw_pointer", |b| {
            b.iter(|| black_box(medium.encode_be_to_raw_stack()))
        });
    }

    // Large struct benchmarks
    let large = create_large_struct();
    group.throughput(Throughput::Bytes(LargeStruct::field_size() as u64));

    group.bench_function("large_to_be_bytes", |b| {
        b.iter(|| black_box(large.to_be_bytes()))
    });

    group.bench_function("large_to_be_bytes_buf", |b| {
        b.iter(|| black_box(large.to_be_bytes_buf()))
    });

    if LargeStruct::supports_raw_pointer_encoding() {
        group.bench_function("large_raw_pointer", |b| {
            b.iter(|| black_box(large.encode_be_to_raw_stack()))
        });
    }

    group.finish();
}

fn bench_bit_field_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bit_field_operations");

    let bit_field = create_bit_field_struct();
    group.throughput(Throughput::Bytes(BitFieldStruct::field_size() as u64));

    group.bench_function("bit_field_serialization", |b| {
        b.iter(|| black_box(bit_field.to_be_bytes()))
    });

    group.bench_function("bit_field_bytes_buf", |b| {
        b.iter(|| black_box(bit_field.to_be_bytes_buf()))
    });

    // Test deserialization
    let serialized = bit_field.to_be_bytes();
    group.bench_function("bit_field_deserialization", |b| {
        b.iter(|| {
            let (decoded, _) = BitFieldStruct::try_from_be_bytes(black_box(&serialized)).unwrap();
            black_box(decoded);
        })
    });

    group.finish();
}

fn bench_vector_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_operations");

    // Fixed size vector
    let vector_struct = create_vector_struct();
    group.throughput(Throughput::Bytes(VectorStruct::field_size() as u64));

    group.bench_function("fixed_vector_serialization", |b| {
        b.iter(|| black_box(vector_struct.to_be_bytes()))
    });

    // Dynamic vector - different sizes
    for size in [1, 4, 16, 64, 255].iter() {
        let dynamic = create_dynamic_vector_struct(*size);
        group.bench_function(&format!("dynamic_vector_size_{}", size), |b| {
            b.iter(|| black_box(dynamic.to_be_bytes()))
        });
    }

    group.finish();
}

fn bench_endianness_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("endianness_comparison");

    let medium = create_medium_struct();
    group.throughput(Throughput::Bytes(MediumStruct::field_size() as u64));

    group.bench_function("big_endian_serialization", |b| {
        b.iter(|| black_box(medium.to_be_bytes()))
    });

    group.bench_function("little_endian_serialization", |b| {
        b.iter(|| black_box(medium.to_le_bytes()))
    });

    // Deserialization comparison
    let be_bytes = medium.to_be_bytes();
    let le_bytes = medium.to_le_bytes();

    group.bench_function("big_endian_deserialization", |b| {
        b.iter(|| {
            let (decoded, _) = MediumStruct::try_from_be_bytes(black_box(&be_bytes)).unwrap();
            black_box(decoded);
        })
    });

    group.bench_function("little_endian_deserialization", |b| {
        b.iter(|| {
            let (decoded, _) = MediumStruct::try_from_le_bytes(black_box(&le_bytes)).unwrap();
            black_box(decoded);
        })
    });

    group.finish();
}

fn bench_mixed_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_complexity");

    // Test different payload sizes
    for payload_size in [0, 16, 64, 256, 1024].iter() {
        let mixed = create_mixed_struct(*payload_size);
        group.bench_function(&format!("mixed_payload_{}", payload_size), |b| {
            b.iter(|| black_box(mixed.to_be_bytes()))
        });
    }

    group.finish();
}

fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");

    // Batch serialization of multiple structs
    let structs: Vec<SmallStruct> = (0..1000).map(|_| create_small_struct()).collect();
    group.throughput(Throughput::Elements(1000));

    group.bench_function("batch_serialization_vec", |b| {
        b.iter(|| {
            let results: Vec<Vec<u8>> = structs.iter().map(|s| s.to_be_bytes()).collect();
            black_box(results);
        })
    });

    group.bench_function("batch_serialization_bytes", |b| {
        b.iter(|| {
            let results: Vec<_> = structs.iter().map(|s| s.to_be_bytes_buf()).collect();
            black_box(results);
        })
    });

    group.bench_function("batch_single_buffer", |b| {
        b.iter_batched(
            || BytesMut::with_capacity(SmallStruct::field_size() * 1000),
            |mut buf| {
                for s in &structs {
                    s.encode_be_to(&mut buf).unwrap();
                }
                black_box(buf);
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_serialization_methods,
    bench_bit_field_operations,
    bench_vector_operations,
    bench_endianness_comparison,
    bench_mixed_complexity,
    bench_batch_operations
);
criterion_main!(benches);
