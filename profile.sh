#!/bin/bash

# BeBytes Performance Profiling Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== BeBytes Performance Profiling Script ===${NC}"
echo ""

# Check if we're on macOS or Linux
if [[ "$OSTYPE" == "darwin"* ]]; then
    PROFILER="instruments"
    echo -e "${YELLOW}Detected macOS - will use Instruments for profiling${NC}"
else
    PROFILER="perf"
    echo -e "${YELLOW}Detected Linux - will use perf for profiling${NC}"
fi

# Build the benchmark in release mode with debug symbols
echo -e "${GREEN}Building benchmark suite...${NC}"
cargo build --bin performance_benchmark --release

# Create profiling output directory
mkdir -p profiling_results
cd profiling_results

# Function to run profiling with different tools
run_profiling() {
    local tool=$1
    local description=$2
    
    echo -e "${GREEN}=== ${description} ===${NC}"
    
    case $tool in
        "perf-record")
            echo "Running perf record for CPU profiling..."
            perf record -g --call-graph=dwarf -o perf.data ../target/release/performance_benchmark
            echo "Generating perf report..."
            perf report -i perf.data > perf_report.txt
            echo "CPU profiling complete. Report saved to perf_report.txt"
            ;;
        "perf-stat")
            echo "Running perf stat for performance counters..."
            perf stat -e cache-misses,cache-references,instructions,cycles,branch-misses,branches \
                ../target/release/performance_benchmark 2> perf_stat.txt
            echo "Performance counters saved to perf_stat.txt"
            ;;
        "valgrind-callgrind")
            echo "Running valgrind callgrind for call graph analysis..."
            valgrind --tool=callgrind --callgrind-out-file=callgrind.out ../target/release/performance_benchmark
            echo "Call graph analysis complete. Output saved to callgrind.out"
            echo "Use 'callgrind_annotate callgrind.out' or kcachegrind to analyze"
            ;;
        "valgrind-massif")
            echo "Running valgrind massif for memory profiling..."
            valgrind --tool=massif --massif-out-file=massif.out ../target/release/performance_benchmark
            echo "Memory profiling complete. Output saved to massif.out"
            echo "Use 'ms_print massif.out' to analyze memory usage"
            ;;
        "instruments-time")
            echo "Running Instruments Time Profiler..."
            instruments -t "Time Profiler" -D instruments_time.trace ../target/release/performance_benchmark
            echo "Time profiling complete. Open instruments_time.trace with Instruments"
            ;;
        "instruments-allocations")
            echo "Running Instruments Allocations profiler..."
            instruments -t "Allocations" -D instruments_allocations.trace ../target/release/performance_benchmark
            echo "Allocations profiling complete. Open instruments_allocations.trace with Instruments"
            ;;
        "cargo-flamegraph")
            echo "Running cargo flamegraph..."
            if command -v cargo-flamegraph &> /dev/null; then
                cargo flamegraph --bin performance_benchmark --output flamegraph.svg
                echo "Flamegraph generated: flamegraph.svg"
            else
                echo "cargo-flamegraph not found. Install with: cargo install flamegraph"
            fi
            ;;
    esac
    echo ""
}

# Menu for profiling options
show_menu() {
    echo -e "${YELLOW}Choose profiling tool:${NC}"
    echo "1) perf record (CPU profiling)"
    echo "2) perf stat (performance counters)"
    echo "3) valgrind callgrind (call graph)"
    echo "4) valgrind massif (memory profiling)"
    echo "5) cargo flamegraph (flame graph)"
    echo "6) All Linux tools"
    echo "7) Instruments Time Profiler (macOS)"
    echo "8) Instruments Allocations (macOS)"
    echo "9) All macOS tools"
    echo "10) Quick run (just timing)"
    echo "0) Exit"
    echo ""
}

# Handle user selection
handle_selection() {
    local choice=$1
    
    case $choice in
        1)
            run_profiling "perf-record" "CPU Profiling with perf"
            ;;
        2)
            run_profiling "perf-stat" "Performance Counters with perf"
            ;;
        3)
            run_profiling "valgrind-callgrind" "Call Graph Analysis with Valgrind"
            ;;
        4)
            run_profiling "valgrind-massif" "Memory Profiling with Valgrind"
            ;;
        5)
            run_profiling "cargo-flamegraph" "Flame Graph Generation"
            ;;
        6)
            run_profiling "perf-record" "CPU Profiling with perf"
            run_profiling "perf-stat" "Performance Counters with perf"
            run_profiling "valgrind-callgrind" "Call Graph Analysis with Valgrind"
            run_profiling "valgrind-massif" "Memory Profiling with Valgrind"
            run_profiling "cargo-flamegraph" "Flame Graph Generation"
            ;;
        7)
            run_profiling "instruments-time" "Time Profiling with Instruments"
            ;;
        8)
            run_profiling "instruments-allocations" "Allocations Profiling with Instruments"
            ;;
        9)
            run_profiling "instruments-time" "Time Profiling with Instruments"
            run_profiling "instruments-allocations" "Allocations Profiling with Instruments"
            ;;
        10)
            echo -e "${GREEN}Running quick timing test...${NC}"
            time ../target/release/performance_benchmark
            ;;
        0)
            echo -e "${GREEN}Exiting...${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}Invalid option${NC}"
            return 1
            ;;
    esac
}

# Main menu loop
while true; do
    show_menu
    read -p "Enter your choice: " choice
    handle_selection $choice
    
    if [[ $choice != 0 ]]; then
        echo -e "${YELLOW}Press Enter to continue...${NC}"
        read
    fi
done