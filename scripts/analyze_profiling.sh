#!/bin/bash
# Script to analyze profiling data from benchmarks
#
# The benchmarks generate flamegraph SVG files that visualize where time is spent.
# This script helps locate and view these files.

set -e

CRITERION_DIR="target/criterion"
PROFILE_DIR="$CRITERION_DIR/profile"

echo "=== Benchmark Profiling Analysis ==="
echo ""

# Check if criterion directory exists
if [ ! -d "$CRITERION_DIR" ]; then
    echo "Error: No benchmark results found in $CRITERION_DIR"
    echo "Run benchmarks first: cargo bench --bench cross_store_comparison --features native"
    exit 1
fi

echo "Finding flamegraph files..."
echo ""

# Find all flamegraph files
FLAMEGRAPHS=$(find "$CRITERION_DIR" -name "flamegraph.svg" 2>/dev/null || true)

if [ -z "$FLAMEGRAPHS" ]; then
    echo "No flamegraph files found."
    echo ""
    echo "Flamegraphs are generated in the benchmark output directories."
    echo "Look for files like: $CRITERION_DIR/<benchmark_name>/<variant>/profile/flamegraph.svg"
    exit 0
fi

echo "Found flamegraph files:"
echo ""

# List all flamegraphs with their paths
echo "$FLAMEGRAPHS" | while read -r file; do
    # Extract benchmark name from path
    benchmark_name=$(echo "$file" | sed "s|$CRITERION_DIR/||" | sed 's|/profile/flamegraph.svg||')
    size=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null || echo "unknown")
    echo "  📊 $benchmark_name"
    echo "     Path: $file"
    echo "     Size: $size bytes"
    echo ""
done

echo "=== How to View Flamegraphs ==="
echo ""
echo "1. Open SVG files directly in a web browser:"
echo "   firefox target/criterion/<benchmark_name>/profile/flamegraph.svg"
echo ""
echo "2. View all flamegraphs for a specific benchmark:"
echo "   find target/criterion/cross_store_insert -name 'flamegraph.svg' -exec firefox {} +"
echo ""
echo "3. Copy to docs for documentation:"
echo "   mkdir -p docs/profiling"
echo "   find target/criterion -name 'flamegraph.svg' -exec cp {} docs/profiling/ \\;"
echo ""

echo "=== Performance Hotspots to Look For ==="
echo ""
echo "In the flamegraphs, look for:"
echo "  • Wide bars = Functions taking significant time"
echo "  • bincode operations = Serialization overhead"
echo "  • redb transaction operations = Database overhead"
echo "  • Memory allocation = alloc/dealloc calls"
echo "  • Lock contention = mutex/rwlock operations"
echo ""

echo "=== Recommended Flamegraphs to Check ==="
echo ""
echo "For wrapper overhead analysis:"
echo "  • wrapper_redb_loop vs wrapper_redb_bulk"
echo "  • wrapper_redb vs raw_redb"
echo ""
echo "For zerocopy overhead analysis:"
echo "  • zerocopy_redb vs raw_redb"
echo "  • zerocopy_redb_bulk vs zerocopy_redb"
echo ""
