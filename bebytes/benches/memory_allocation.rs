use bebytes::BeBytes;
use bytes::BytesMut;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::hint::black_box;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// ============ Memory Tracking Allocator ============

struct TrackingAllocator {
    allocations: AtomicUsize,
    deallocations: AtomicUsize,
    total_allocated: AtomicUsize,
    peak_allocated: AtomicUsize,
    current_allocated: AtomicUsize,
}

impl TrackingAllocator {
    const fn new() -> Self {
        Self {
            allocations: AtomicUsize::new(0),
            deallocations: AtomicUsize::new(0),
            total_allocated: AtomicUsize::new(0),
            peak_allocated: AtomicUsize::new(0),
            current_allocated: AtomicUsize::new(0),
        }
    }

    fn reset(&self) {
        self.allocations.store(0, Ordering::SeqCst);
        self.deallocations.store(0, Ordering::SeqCst);
        self.total_allocated.store(0, Ordering::SeqCst);
        self.peak_allocated.store(0, Ordering::SeqCst);
        self.current_allocated.store(0, Ordering::SeqCst);
    }

    fn stats(&self) -> AllocationStats {
        AllocationStats {
            allocations: self.allocations.load(Ordering::SeqCst),
            deallocations: self.deallocations.load(Ordering::SeqCst),
            total_allocated: self.total_allocated.load(Ordering::SeqCst),
            peak_allocated: self.peak_allocated.load(Ordering::SeqCst),
            current_allocated: self.current_allocated.load(Ordering::SeqCst),
        }
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            self.allocations.fetch_add(1, Ordering::SeqCst);
            self.total_allocated
                .fetch_add(layout.size(), Ordering::SeqCst);
            let current = self
                .current_allocated
                .fetch_add(layout.size(), Ordering::SeqCst)
                + layout.size();

            // Update peak if necessary
            let mut peak = self.peak_allocated.load(Ordering::SeqCst);
            while current > peak {
                match self.peak_allocated.compare_exchange_weak(
                    peak,
                    current,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => break,
                    Err(new_peak) => peak = new_peak,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        self.deallocations.fetch_add(1, Ordering::SeqCst);
        self.current_allocated
            .fetch_sub(layout.size(), Ordering::SeqCst);
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator::new();

#[derive(Debug, Clone)]
struct AllocationStats {
    allocations: usize,
    #[allow(dead_code)]
    deallocations: usize,
    total_allocated: usize,
    peak_allocated: usize,
    #[allow(dead_code)]
    current_allocated: usize,
}

impl AllocationStats {
    fn allocations_per_op(&self, operations: usize) -> f64 {
        self.allocations as f64 / operations as f64
    }

    fn bytes_per_op(&self, operations: usize) -> f64 {
        self.total_allocated as f64 / operations as f64
    }

    fn efficiency_ratio(&self) -> f64 {
        if self.total_allocated == 0 {
            0.0
        } else {
            1.0 - (self.peak_allocated as f64 / self.total_allocated as f64)
        }
    }
}

// Custom helper for memory statistics reporting
fn print_memory_stats(label: &str, stats: &AllocationStats, operations: usize) {
    eprintln!(
        "{} - Allocations: {}, Total bytes: {}, Allocs/op: {:.2}, Bytes/op: {:.2}",
        label,
        stats.allocations,
        stats.total_allocated,
        stats.allocations_per_op(operations),
        stats.bytes_per_op(operations)
    );
}

// ============ Test Structures ============

#[derive(BeBytes, Clone, Debug)]
struct TinyStruct {
    a: u16,
    b: u16,
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
    data: [u8; 256],
    checksum: u64,
}

#[derive(BeBytes, Clone, Debug)]
struct VectorStruct {
    header: u32,
    #[With(size(64))]
    data: Vec<u8>,
    footer: u16,
}

#[derive(BeBytes, Clone, Debug)]
struct DynamicStruct {
    length: u16,
    #[FromField(length)]
    payload: Vec<u8>,
    checksum: u32,
}

// ============ Helper Functions ============

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
        data: [0xAA; 256],
        checksum: 0xDEADBEEFCAFEBABE,
    }
}

#[allow(dead_code)]
fn create_vector_struct() -> VectorStruct {
    VectorStruct {
        header: 0x12345678,
        data: vec![0x42; 64],
        footer: 0xABCD,
    }
}

fn create_dynamic_struct(size: u16) -> DynamicStruct {
    DynamicStruct {
        length: size,
        payload: vec![0x42; size as usize],
        checksum: 0xDEADBEEF,
    }
}

// ============ Memory Allocation Benchmarks ============

fn bench_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_patterns");

    let medium = create_medium_struct();

    // Measure allocations for Vec approach
    group.bench_function("vec_allocations", |b| {
        b.iter_custom(|iters| {
            ALLOCATOR.reset();
            let start = std::time::Instant::now();

            for _ in 0..iters {
                let result = medium.to_be_bytes();
                black_box(result);
            }

            let duration = start.elapsed();
            let stats = ALLOCATOR.stats();
            print_memory_stats("Vec approach", &stats, iters as usize);

            duration
        });
    });

    // Measure allocations for Bytes approach
    group.bench_function("bytes_allocations", |b| {
        b.iter_custom(|iters| {
            ALLOCATOR.reset();
            let start = std::time::Instant::now();

            for _ in 0..iters {
                let result = medium.to_be_bytes_buf();
                black_box(result);
            }

            let duration = start.elapsed();
            let stats = ALLOCATOR.stats();
            print_memory_stats("Bytes approach", &stats, iters as usize);

            duration
        });
    });

    // Measure allocations for direct BufMut approach
    group.bench_function("bufmut_allocations", |b| {
        b.iter_custom(|iters| {
            ALLOCATOR.reset();
            let start = std::time::Instant::now();

            for _ in 0..iters {
                let mut buf = BytesMut::with_capacity(MediumStruct::field_size());
                medium.encode_be_to(&mut buf).unwrap();
                black_box(buf);
            }

            let duration = start.elapsed();
            let stats = ALLOCATOR.stats();
            print_memory_stats("BufMut approach", &stats, iters as usize);

            duration
        });
    });

    group.finish();
}

fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    // Test different vector sizes
    for size in [16, 64, 256, 1024, 4096].iter() {
        let dynamic = create_dynamic_struct(*size);

        group.bench_function(&format!("dynamic_size_{}_vec", size), |b| {
            b.iter_custom(|iters| {
                ALLOCATOR.reset();
                let start = std::time::Instant::now();

                for _ in 0..iters {
                    let result = dynamic.to_be_bytes();
                    black_box(result);
                }

                let duration = start.elapsed();
                let stats = ALLOCATOR.stats();

                // Calculate overhead ratio (allocated vs actual struct size)
                let expected_size = dynamic.to_be_bytes().len();
                let overhead_ratio = stats.bytes_per_op(iters as usize) / expected_size as f64;

                eprintln!(
                    "Size {} Vec - Expected: {} bytes, Overhead ratio: {:.2}x",
                    size, expected_size, overhead_ratio
                );
                print_memory_stats(&format!("Size {} Vec", size), &stats, iters as usize);

                duration
            });
        });

        group.bench_function(&format!("dynamic_size_{}_bytes", size), |b| {
            b.iter_custom(|iters| {
                ALLOCATOR.reset();
                let start = std::time::Instant::now();

                for _ in 0..iters {
                    let result = dynamic.to_be_bytes_buf();
                    black_box(result);
                }

                let duration = start.elapsed();
                let stats = ALLOCATOR.stats();

                let expected_size = dynamic.to_be_bytes().len();
                let overhead_ratio = stats.bytes_per_op(iters as usize) / expected_size as f64;

                eprintln!(
                    "Size {} Bytes - Expected: {} bytes, Overhead ratio: {:.2}x",
                    size, expected_size, overhead_ratio
                );
                print_memory_stats(&format!("Size {} Bytes", size), &stats, iters as usize);

                duration
            });
        });
    }

    group.finish();
}

fn bench_memory_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_reuse");

    let medium = create_medium_struct();

    // Test buffer reuse patterns
    group.bench_function("buffer_reuse", |b| {
        b.iter_batched(
            || BytesMut::with_capacity(MediumStruct::field_size() * 100),
            |mut buf| {
                ALLOCATOR.reset();

                for _ in 0..100 {
                    buf.clear();
                    medium.encode_be_to(&mut buf).unwrap();
                    black_box(&buf[..]);
                }

                let stats = ALLOCATOR.stats();
                print_memory_stats("Buffer reuse", &stats, 100);
            },
            BatchSize::SmallInput,
        );
    });

    // Test allocation per operation
    group.bench_function("no_reuse", |b| {
        b.iter_custom(|iters| {
            ALLOCATOR.reset();
            let start = std::time::Instant::now();

            for _ in 0..iters {
                let mut buf = BytesMut::with_capacity(MediumStruct::field_size());
                medium.encode_be_to(&mut buf).unwrap();
                black_box(buf);
            }

            let duration = start.elapsed();
            let stats = ALLOCATOR.stats();
            print_memory_stats("No reuse", &stats, iters as usize);

            duration
        });
    });

    group.finish();
}

fn bench_peak_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("peak_memory");

    // Test peak memory for different workloads
    let structs: Vec<LargeStruct> = (0..1000).map(|_| create_large_struct()).collect();

    group.bench_function("batch_peak_memory", |b| {
        b.iter_custom(|iters| {
            ALLOCATOR.reset();
            let start = std::time::Instant::now();

            for _ in 0..iters {
                let results: Vec<Vec<u8>> = structs
                    .iter()
                    .take(100) // Limit to 100 to avoid excessive memory usage
                    .map(|s| s.to_be_bytes())
                    .collect();
                black_box(results);
            }

            let duration = start.elapsed();
            let stats = ALLOCATOR.stats();
            eprintln!(
                "Batch processing - Peak: {} bytes, Total: {} bytes, Efficiency: {:.2}",
                stats.peak_allocated,
                stats.total_allocated,
                stats.efficiency_ratio()
            );

            duration
        });
    });

    group.bench_function("streaming_peak_memory", |b| {
        b.iter_custom(|iters| {
            ALLOCATOR.reset();
            let start = std::time::Instant::now();

            for _ in 0..iters {
                for s in structs.iter().take(100) {
                    let result = s.to_be_bytes();
                    black_box(result);
                    // Result is dropped immediately, reducing peak memory
                }
            }

            let duration = start.elapsed();
            let stats = ALLOCATOR.stats();
            eprintln!(
                "Streaming processing - Peak: {} bytes, Total: {} bytes, Efficiency: {:.2}",
                stats.peak_allocated,
                stats.total_allocated,
                stats.efficiency_ratio()
            );

            duration
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_allocation_patterns,
    bench_memory_efficiency,
    bench_memory_reuse,
    bench_peak_memory_usage
);
criterion_main!(benches);
