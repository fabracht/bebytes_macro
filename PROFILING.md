# BeBytes Performance Profiling Guide

This guide explains how to profile the BeBytes runtime performance using the comprehensive benchmark suite.

## Quick Start

1. **Run the benchmark suite**:
   ```bash
   ./profile.sh
   ```
   
2. **Choose your profiling tool** from the menu:
   - Option 10: Quick run (just timing) - fastest way to get basic performance data
   - Option 6: All Linux tools - comprehensive analysis on Linux
   - Option 9: All macOS tools - comprehensive analysis on macOS

## Benchmark Structure

The benchmark suite (`bebytes/bin/performance_benchmark.rs`) includes:

### 1. Primitive Serialization (1M iterations)
- **U8 serialization**: Simple bit field operations
- **U16 serialization**: Cross-byte bit field operations  
- **U32 serialization**: Multi-byte bit field operations
- **Deserialization**: Parsing operations

### 2. Bit Field Operations (500K iterations)
- **Complex bit fields**: Mixed bit sizes with u32 fields
- **Cross-byte operations**: Bit fields spanning byte boundaries
- **Round-trip operations**: Serialization + deserialization

### 3. Enum Operations (1M iterations)
- **Regular enums**: Standard enum serialization
- **Auto-sized enums**: Enums with `#[bits()]` attribute
- **Flag enums**: Bitwise flag operations

### 4. Vector Operations (100K iterations)
- **Fixed-size vectors**: `#[With(size(n))]` attribute
- **Dynamic vectors**: `#[FromField(field_name)]` attribute
- **Vector deserialization**: Parsing with size constraints

### 5. Nested Structures (50K iterations)
- **Complex nesting**: Multiple levels of struct nesting
- **Mixed types**: Combination of bit fields, enums, and vectors
- **Memory allocation**: Dynamic structure creation

### 6. Array Operations (200K iterations)
- **Fixed-size arrays**: `[T; N]` serialization
- **Multi-dimensional**: Arrays of different sizes
- **Custom types**: Arrays of custom structs

### 7. Mixed Scenarios (10K iterations)
- **Real-world structures**: Complex packet-like structures
- **Memory stress test**: Large dynamic allocations
- **DNS-like parsing**: Variable-length segments

## Profiling Tools

### Linux Tools

#### perf
```bash
# CPU profiling
perf record -g --call-graph=dwarf ./target/release/macro_test
perf report

# Performance counters
perf stat -e cache-misses,cache-references,instructions,cycles,branch-misses,branches ./target/release/macro_test
```

#### Valgrind
```bash
# Call graph analysis
valgrind --tool=callgrind --callgrind-out-file=callgrind.out ./target/release/macro_test
callgrind_annotate callgrind.out

# Memory profiling
valgrind --tool=massif --massif-out-file=massif.out ./target/release/macro_test
ms_print massif.out
```

#### Flamegraph
```bash
# Install if needed
cargo install flamegraph

# Generate flame graph
cargo flamegraph --bin macro_test --output flamegraph.svg
```

### macOS Tools

#### Instruments
```bash
# Time profiling
instruments -t "Time Profiler" -D time_profile.trace ./target/release/macro_test

# Memory profiling
instruments -t "Allocations" -D allocations.trace ./target/release/macro_test
```

## Key Performance Metrics

### Current Benchmark Results (Example)
- **Simple operations**: 0.28-0.40 ns/op (U8, enum serialization)
- **Bit field operations**: 0.30-33.05 ns/op (cross-byte operations are slower)
- **Vector operations**: 18.88-107 ns/op (serialization more expensive than deserialization)
- **Nested structures**: 1.34-172.38 ns/op (serialization more expensive)
- **Complex scenarios**: 569.73-602.99 ns/op (real-world complexity)

### What to Look For

1. **CPU Hotspots**:
   - Bit manipulation functions
   - Vector allocation and resizing
   - Endianness conversion
   - Memory copying operations

2. **Memory Patterns**:
   - Vec::with_capacity usage
   - Buffer resizing during serialization
   - Heap allocations in complex structures
   - Memory fragmentation

3. **Cache Performance**:
   - Cache misses during bit operations
   - Memory access patterns
   - Data locality in nested structures

4. **Optimization Opportunities**:
   - Redundant operations
   - Unnecessary allocations
   - Inefficient bit manipulation
   - Buffer management issues

## Analysis Workflow

1. **Baseline measurement**: Run quick timing to establish baseline
2. **CPU profiling**: Identify hotspots with perf/Instruments
3. **Memory profiling**: Check allocation patterns with valgrind/Instruments
4. **Cache analysis**: Look for cache misses and memory access patterns
5. **Flame graph**: Visualize call stack and time distribution

## Output Files

The profiling script creates a `profiling_results/` directory with:
- `perf_report.txt`: CPU profiling report
- `perf_stat.txt`: Performance counters
- `callgrind.out`: Call graph data
- `massif.out`: Memory usage data
- `flamegraph.svg`: Flame graph visualization
- `*.trace`: Instruments trace files (macOS)

## Tips for Effective Profiling

1. **Use release builds**: Always profile optimized code
2. **Multiple runs**: Profile multiple times to account for variance
3. **Isolate scenarios**: Focus on specific benchmark functions
4. **Compare changes**: Profile before and after optimizations
5. **Consider real workloads**: Supplement with real-world usage patterns

## Next Steps

After profiling, use the results to:
1. Identify the most expensive operations
2. Focus optimization efforts on high-impact areas
3. Validate improvements with before/after comparisons
4. Document performance characteristics for users