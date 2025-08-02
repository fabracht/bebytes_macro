use bebytes::BeBytes;
use bytes::BytesMut;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use std::hint::black_box;

// ============ Test Structures for Cache Analysis ============

#[derive(BeBytes, Clone, Debug)]
struct CacheFriendlyStruct {
    // 64 bytes - fits in a single cache line
    data: [u8; 60],
    checksum: u32,
}

#[derive(BeBytes, Clone, Debug)]
struct CacheUnfriendlyStruct {
    // 128 bytes - spans two cache lines
    data: [u8; 124],
    checksum: u32,
}

#[derive(BeBytes, Clone, Debug)]
struct SmallStruct {
    // 8 bytes - multiple fit in one cache line
    a: u32,
    b: u32,
}

#[derive(BeBytes, Clone, Debug)]
struct BitFieldStruct {
    #[bits(4)]
    version: u8,
    #[bits(4)]
    flags: u8,
    #[bits(8)]
    type_field: u8,
    #[bits(16)]
    length: u16,
    payload: u32,
}

#[derive(BeBytes, Clone, Debug)]
struct PaddedStruct {
    // Test alignment and padding effects
    byte_field: u8,
    // 3 bytes padding
    int_field: u32,
    // 0 bytes padding
    long_field: u64,
}

#[derive(BeBytes, Clone, Debug)]
struct VectorStruct {
    header: u32,
    #[With(size(256))]
    data: Vec<u8>,
    footer: u32,
}

// ============ Helper Functions ============

fn create_cache_friendly_data(size: usize) -> Vec<CacheFriendlyStruct> {
    (0..size)
        .map(|i| CacheFriendlyStruct {
            data: [i as u8; 60],
            checksum: i as u32,
        })
        .collect()
}

fn create_cache_unfriendly_data(size: usize) -> Vec<CacheUnfriendlyStruct> {
    (0..size)
        .map(|i| CacheUnfriendlyStruct {
            data: [i as u8; 124],
            checksum: i as u32,
        })
        .collect()
}

fn create_small_data(size: usize) -> Vec<SmallStruct> {
    (0..size)
        .map(|i| SmallStruct {
            a: i as u32,
            b: (i * 2) as u32,
        })
        .collect()
}

fn create_scattered_data(size: usize) -> Vec<Box<SmallStruct>> {
    // Allocate scattered across heap to test cache locality
    (0..size)
        .map(|i| {
            Box::new(SmallStruct {
                a: i as u32,
                b: (i * 2) as u32,
            })
        })
        .collect()
}

// ============ Cache Line Efficiency Benchmarks ============

fn bench_cache_line_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_line_efficiency");

    let cache_friendly = create_cache_friendly_data(1000);
    let cache_unfriendly = create_cache_unfriendly_data(1000);

    group.throughput(Throughput::Elements(1000));

    // Sequential access - cache friendly
    group.bench_function("cache_friendly_sequential", |b| {
        b.iter(|| {
            for item in &cache_friendly {
                let result = item.to_be_bytes();
                black_box(result);
            }
        })
    });

    // Sequential access - cache unfriendly
    group.bench_function("cache_unfriendly_sequential", |b| {
        b.iter(|| {
            for item in &cache_unfriendly {
                let result = item.to_be_bytes();
                black_box(result);
            }
        })
    });

    // Random access patterns
    let indices: Vec<usize> = (0..1000).rev().collect(); // Reverse order for poor locality

    group.bench_function("cache_friendly_random", |b| {
        b.iter(|| {
            for &idx in &indices {
                let result = cache_friendly[idx].to_be_bytes();
                black_box(result);
            }
        })
    });

    group.bench_function("cache_unfriendly_random", |b| {
        b.iter(|| {
            for &idx in &indices {
                let result = cache_unfriendly[idx].to_be_bytes();
                black_box(result);
            }
        })
    });

    group.finish();
}

fn bench_memory_layout_effects(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_layout");

    let contiguous = create_small_data(1000);
    let scattered = create_scattered_data(1000);

    group.throughput(Throughput::Elements(1000));

    // Contiguous memory layout
    group.bench_function("contiguous_layout", |b| {
        b.iter(|| {
            for item in &contiguous {
                let result = item.to_be_bytes();
                black_box(result);
            }
        })
    });

    // Scattered memory layout (heap allocations)
    group.bench_function("scattered_layout", |b| {
        b.iter(|| {
            for item in &scattered {
                let result = item.to_be_bytes();
                black_box(result);
            }
        })
    });

    group.finish();
}

fn bench_prefetching_effects(c: &mut Criterion) {
    let mut group = c.benchmark_group("prefetching");

    let data = create_cache_friendly_data(1000);

    group.throughput(Throughput::Elements(1000));

    // Without prefetching
    group.bench_function("no_prefetch", |b| {
        b.iter(|| {
            for item in &data {
                let result = item.to_be_bytes();
                black_box(result);
            }
        })
    });

    // With software prefetching
    group.bench_function("with_prefetch", |b| {
        b.iter(|| {
            for (i, item) in data.iter().enumerate() {
                // Prefetch next item if available
                if i + 1 < data.len() {
                    black_box(&data[i + 1]);
                }

                let result = item.to_be_bytes();
                black_box(result);
            }
        })
    });

    group.finish();
}

fn bench_data_structure_packing(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_packing");

    let bit_fields: Vec<BitFieldStruct> = (0..1000)
        .map(|i| BitFieldStruct {
            version: (i % 16) as u8,
            flags: (i % 16) as u8,
            type_field: (i % 256) as u8,
            length: (i % 65536) as u16,
            payload: i as u32,
        })
        .collect();

    let padded: Vec<PaddedStruct> = (0..1000)
        .map(|i| PaddedStruct {
            byte_field: (i % 256) as u8,
            int_field: i as u32,
            long_field: (i as u64) << 32,
        })
        .collect();

    group.throughput(Throughput::Elements(1000));

    group.bench_function("bit_field_packed", |b| {
        b.iter(|| {
            for item in &bit_fields {
                let result = item.to_be_bytes();
                black_box(result);
            }
        })
    });

    group.bench_function("naturally_padded", |b| {
        b.iter(|| {
            for item in &padded {
                let result = item.to_be_bytes();
                black_box(result);
            }
        })
    });

    group.finish();
}

fn bench_buffer_locality(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_locality");

    let small_structs = create_small_data(1000);

    group.throughput(Throughput::Elements(1000));

    // Individual allocations (poor locality)
    group.bench_function("individual_buffers", |b| {
        b.iter(|| {
            let results: Vec<Vec<u8>> = small_structs.iter().map(|s| s.to_be_bytes()).collect();
            black_box(results);
        })
    });

    // Single large buffer (good locality)
    group.bench_function("single_buffer", |b| {
        b.iter_batched(
            || BytesMut::with_capacity(SmallStruct::field_size() * 1000),
            |mut buf| {
                for s in &small_structs {
                    s.encode_be_to(&mut buf).unwrap();
                }
                black_box(buf);
            },
            BatchSize::SmallInput,
        )
    });

    // Pre-allocated buffer reuse
    group.bench_function("buffer_reuse", |b| {
        b.iter_batched(
            || BytesMut::with_capacity(SmallStruct::field_size()),
            |mut buf| {
                for s in &small_structs {
                    buf.clear();
                    s.encode_be_to(&mut buf).unwrap();
                    black_box(&buf[..]);
                }
                black_box(buf);
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_vector_access_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_access");

    let vector_structs: Vec<VectorStruct> = (0..100)
        .map(|i| VectorStruct {
            header: i as u32,
            data: vec![i as u8; 256],
            footer: (i * 2) as u32,
        })
        .collect();

    group.throughput(Throughput::Elements(100));

    // Sequential vector processing
    group.bench_function("sequential_vectors", |b| {
        b.iter(|| {
            for item in &vector_structs {
                let result = item.to_be_bytes();
                black_box(result);
            }
        })
    });

    // Process vectors in chunks to improve cache locality
    group.bench_function("chunked_vectors", |b| {
        b.iter(|| {
            for chunk in vector_structs.chunks(10) {
                for item in chunk {
                    let result = item.to_be_bytes();
                    black_box(result);
                }
            }
        })
    });

    group.finish();
}

fn bench_instruction_cache_effects(c: &mut Criterion) {
    let mut group = c.benchmark_group("instruction_cache");

    let mixed_structs = create_small_data(1000);

    group.throughput(Throughput::Elements(1000));

    // Monomorphic dispatch (good for instruction cache)
    group.bench_function("monomorphic", |b| {
        b.iter(|| {
            for item in &mixed_structs {
                let result = item.to_be_bytes();
                black_box(result);
            }
        })
    });

    // Simulate polymorphic dispatch (harder on instruction cache)
    group.bench_function("polymorphic_simulation", |b| {
        b.iter(|| {
            for (i, item) in mixed_structs.iter().enumerate() {
                let result = match i % 3 {
                    0 => item.to_be_bytes(),
                    1 => item.to_le_bytes(),
                    _ => item.to_be_bytes(),
                };
                black_box(result);
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_cache_line_efficiency,
    bench_memory_layout_effects,
    bench_prefetching_effects,
    bench_data_structure_packing,
    bench_buffer_locality,
    bench_vector_access_patterns,
    bench_instruction_cache_effects
);
criterion_main!(benches);
