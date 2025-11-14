# Benchmark Quick Reference

## Naming Convention Summary

All benchmarks follow the pattern: `{backend}_{api_level}_{method}`

| Component | Values | Meaning |
|-----------|--------|---------|
| **backend** | `sled`, `redb` | Which database |
| **api_level** | `raw`, `wrapper`, `zerocopy` | Abstraction level |
| **method** | `loop`, `bulk`, `txn`, `batch` | Operation pattern |

## Quick Comparison Guide

### Want to see bulk method speedup?
Compare: `redb_wrapper_loop` vs `redb_wrapper_bulk`
- Result: **8-9x faster** for inserts, **2.3x faster** for reads

### Want to see wrapper overhead?
Compare: `redb_wrapper_bulk` vs `redb_raw_txn`
- Result: **~118%** overhead (wrapper is 2.18x slower)

### Want to see zerocopy advantage?
Compare: `redb_zerocopy_txn` vs `redb_raw_loop` (secondary queries)
- Result: **54x faster!** (zerocopy is optimized for this)

### Want to see transaction importance?
Compare: `redb_wrapper_loop` vs `redb_wrapper_bulk`
- Shows cost of N transactions vs 1 transaction

## Performance Hierarchy (1000 items)

### Inserts (fastest to slowest)
1. `redb_raw_txn` - 1.42ms (baseline)
2. `redb_wrapper_bulk` - 3.10ms (+118%)
3. `redb_zerocopy_loop` - 4.34ms (+206%)
4. `redb_wrapper_loop` - 27.3ms (+1822%)

### Reads (fastest to slowest)
1. `redb_raw` - 164µs (baseline)
2. `redb_wrapper_bulk` - 382µs (+133%)
3. `redb_zerocopy_loop` - 692µs (+322%)
4. `redb_wrapper_loop` - 895µs (+446%)

### Secondary Queries - 10 queries (fastest to slowest)
1. **`redb_zerocopy_txn` - 5.41µs (FASTEST!)**
2. `redb_raw_loop` - 291µs (baseline)
3. `redb_wrapper_bulk` - 470µs (+61%)
4. `redb_wrapper_loop` - 1.02ms (+248%)

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench --bench cross_store_comparison --features native

# Run only insert benchmarks
cargo bench --bench cross_store_comparison -- cross_store_insert

# Run only redb benchmarks
cargo bench --bench cross_store_comparison -- redb_

# Run only bulk methods
cargo bench --bench cross_store_comparison -- _bulk

# Generate visualizations
uv run scripts/generate_benchmark_charts.py

# Analyze profiling
./scripts/analyze_profiling.sh
```

## Benchmark Output Files

```
target/criterion/
├── cross_store_insert/
│   ├── redb_wrapper_loop/
│   │   ├── 100/profile/flamegraph.svg
│   │   ├── 1000/profile/flamegraph.svg
│   │   └── ...
│   └── redb_wrapper_bulk/
│       └── ...
└── ...

docs/benchmarks/
├── insert_comparison_bars.png
├── overhead_percentages.png
├── bulk_api_speedup.png
├── raw_vs_zerocopy_comparison.png
└── benchmark_summary.md
```

## Key Takeaways

1. **Always use bulk methods when possible** - 8-9x speedup!
2. **Transaction reuse is critical** - Avoid creating N transactions
3. **ZeroCopy excels at secondary queries** - 54x faster than raw
4. **Wrapper overhead is acceptable** - 118-133% for bulk operations
5. **Profile to find bottlenecks** - Flamegraphs show exactly where time is spent

## Benchmark Categories

| Category | Size | What It Tests |
|----------|------|---------------|
| `cross_store_insert` | [10, 100, 500, 1000, 5000] | Insert scaling |
| `cross_store_get` | 1000 | Read performance |
| `cross_store_bulk_ops` | 1000 | Bulk operations |
| `cross_store_secondary_query` | 10 queries | Secondary key lookups |
| `redb_raw_vs_zerocopy` | Various | Direct overhead measurement |

See [BENCHMARK_NAMING.md](./BENCHMARK_NAMING.md) for complete documentation.
