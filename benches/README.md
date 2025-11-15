# Netabase Store Benchmarks

This directory contains comprehensive benchmarks comparing all storage implementations across multiple dimensions.

## Benchmark Organization

### Dimensions

Benchmarks are organized across these dimensions:

1. **Store Type**: `sled`, `redb`
2. **API Level**: `raw`, `wrapper`, `zerocopy` (redb only)
3. **Operation Mode**: `loop` (individual ops), `batch` (bulk), `transaction` (explicit txn)
4. **Operation Type**: `insert`, `get`, `update`, `delete`, `secondary_query`

### Naming Convention

Benchmark IDs follow this pattern: `{store}_{api}_{mode}`

Examples:
- `sled_raw_loop` - Raw sled API, individual operations
- `sled_raw_batch` - Raw sled API, batched operations
- `sled_wrapper_loop` - Sled wrapper, individual operations (N transactions)
- `sled_wrapper_transaction` - Sled wrapper, explicit transaction (1 transaction)
- `redb_raw_loop` - Raw redb API, loop in single transaction
- `redb_wrapper_loop` - Redb wrapper, individual operations (N transactions)
- `redb_wrapper_batch` - Redb wrapper, `put_many()` (1 transaction)
- `redb_zerocopy_loop` - Zerocopy redb, loop in single transaction

### Comparison Matrix

The benchmarks enable these comparisons:

#### 1. Raw vs Raw (Baseline Store Comparison)
- `sled_raw_loop` vs `redb_raw_loop`
- `sled_raw_batch` vs `redb_raw_loop` (redb always uses transactions)

#### 2. Raw vs Wrapper (Overhead Analysis - Same Store)
- `sled_raw_*` vs `sled_wrapper_*`
- `redb_raw_loop` vs `redb_wrapper_loop`
- `redb_raw_loop` vs `redb_wrapper_batch`

#### 3. Wrapper vs Zerocopy (API Comparison - Redb Only)
- `redb_wrapper_loop` vs `redb_zerocopy_loop`
- `redb_wrapper_batch` vs `redb_zerocopy_loop`

#### 4. Wrapper vs Wrapper (Cross-Store, Same Abstraction Level)
- `sled_wrapper_*` vs `redb_wrapper_*` vs `redb_zerocopy_*`

#### 5. Loop vs Batch vs Transaction (Operation Mode Comparison)
- `*_loop` vs `*_batch` vs `*_transaction` (within same store/API)

## Files

- `cross_store_comparison.rs` - Main comprehensive benchmark file
  - All stores, all APIs, all operation modes
  - Organized into logical benchmark groups by operation type

- `redb_wrapper_overhead.rs` - Focused redb wrapper vs raw analysis
- `redb_zerocopy_overhead.rs` - Focused redb zerocopy vs raw analysis
- `sled_wrapper_overhead.rs` - Focused sled wrapper vs raw analysis

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench --features native

# Run specific benchmark group
cargo bench --features native --bench cross_store_comparison -- cross_store_insert

# Run with filtering
cargo bench --features native -- sled_wrapper

# Generate flamegraphs (requires cargo-flamegraph)
cargo bench --features native -- --profile-time=5
```

## Reading Results

Criterion outputs results in `target/criterion/`:
- HTML reports in `target/criterion/report/index.html`
- Flamegraphs in `target/criterion/{benchmark_name}/profile/flamegraph.svg`

### Interpreting Comparisons

Lower is better for all timings. Key metrics:

- **Throughput**: Operations per second
- **Latency**: Time per operation
- **Overhead**: Wrapper time / Raw time (ideally close to 1.0)

### Expected Results

Based on implementation differences:

1. **Sled vs Redb Raw**: Comparable, architecture-dependent
2. **Wrapper Overhead**: 5-15% for type safety and auto-indexing
3. **Zerocopy vs Wrapper**: 10-50% faster for bulk operations
4. **Loop vs Batch**: Batch 5-10x faster (single transaction vs many)
