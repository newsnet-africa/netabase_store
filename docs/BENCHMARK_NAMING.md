# Benchmark Naming Convention

## Overview

All benchmarks follow a consistent naming scheme to make it easy to compare implementations and understand what each benchmark measures.

## Naming Format

```
{backend}_{api_level}_{method}
```

### Components

1. **backend**: The underlying database
   - `sled` - Sled embedded database
   - `redb` - Redb embedded database

2. **api_level**: The API abstraction level
   - `raw` - Direct database API (baseline for overhead calculation)
   - `wrapper` - Type-safe wrapper API (standard netabase_store API)
   - `zerocopy` - Explicit transaction management API (advanced)

3. **method**: The operation pattern
   - `loop` - Per-item operations in a loop
   - `bulk` - Bulk methods (put_many, get_many, etc.)
   - `txn` - Explicit transaction
   - `batch` - Batch API
   - `read` - Read-only operations
   - `insert` - Insert operations

## Complete Benchmark List

### Insert Benchmarks (cross_store_insert)

Measures insert performance across dataset sizes [10, 100, 500, 1000, 5000].

| Benchmark Name | Description | Transaction Behavior |
|----------------|-------------|---------------------|
| `sled_raw_loop` | Raw Sled API, per-item insert | N transactions (auto-flush) |
| `sled_wrapper_loop` | Wrapper Sled, per-item insert | N transactions |
| `redb_raw_txn` | Raw Redb API, loop in single txn | 1 transaction |
| `redb_wrapper_loop` | Wrapper Redb, per-item insert | N transactions |
| `redb_wrapper_bulk` | Wrapper Redb, `put_many()` | 1 transaction |
| `redb_zerocopy_loop` | ZeroCopy Redb, loop in explicit txn | 1 transaction |
| `redb_zerocopy_bulk` | ZeroCopy Redb, `put_many()` | 1 transaction |

**Key Comparisons:**
- `redb_wrapper_loop` vs `redb_wrapper_bulk` → Shows bulk method speedup (8-9x)
- `redb_wrapper_bulk` vs `redb_raw_txn` → Shows wrapper overhead (~118%)
- `redb_zerocopy_loop` vs `redb_raw_txn` → Shows zerocopy overhead (~200%)

### Get Benchmarks (cross_store_get)

Measures read performance for 1000 items.

| Benchmark Name | Description | Transaction Behavior |
|----------------|-------------|---------------------|
| `sled_raw` | Raw Sled API, per-item get | N transactions |
| `sled_wrapper` | Wrapper Sled, per-item get | N transactions |
| `redb_raw` | Raw Redb API, single txn | 1 transaction |
| `redb_wrapper_loop` | Wrapper Redb, per-item get | N transactions |
| `redb_wrapper_bulk` | Wrapper Redb, `get_many()` | 1 transaction |
| `redb_zerocopy_loop` | ZeroCopy Redb, explicit txn | 1 transaction |

**Key Comparisons:**
- `redb_wrapper_loop` vs `redb_wrapper_bulk` → Shows bulk method speedup (2.3x)
- `redb_wrapper_bulk` vs `redb_raw` → Shows wrapper overhead (~133%)

### Bulk Operations (cross_store_bulk)

Measures bulk insert performance for 1000 items.

| Benchmark Name | Description | Method Used |
|----------------|-------------|-------------|
| `sled_raw_batch` | Raw Sled batch API | `Batch::apply()` |
| `sled_wrapper_loop` | Wrapper Sled, per-item | N `put()` calls |
| `redb_raw_txn` | Raw Redb, single txn | Manual transaction |
| `redb_wrapper_loop` | Wrapper Redb, per-item | N `put()` calls |
| `redb_zerocopy_txn` | ZeroCopy Redb, loop in txn | Explicit transaction |
| `redb_zerocopy_bulk` | ZeroCopy Redb, bulk | `put_many()` in transaction |

### Secondary Key Queries (cross_store_secondary_query)

Measures secondary key query performance for 10 queries.

| Benchmark Name | Description | Transaction Behavior |
|----------------|-------------|---------------------|
| `sled_raw_loop` | Raw Sled, manual index traversal | N transactions |
| `sled_wrapper_loop` | Wrapper Sled, per-query | N `get_by_secondary_key()` |
| `redb_raw_loop` | Raw Redb, manual index | N transactions |
| `redb_wrapper_loop` | Wrapper Redb, per-query | N transactions |
| `redb_wrapper_bulk` | Wrapper Redb, bulk query | 1 transaction |
| `redb_zerocopy_txn` | ZeroCopy Redb, explicit txn | 1 transaction |

**Key Comparisons:**
- `redb_wrapper_loop` vs `redb_wrapper_bulk` → Shows bulk method speedup (2.2x)
- `redb_zerocopy_txn` vs `redb_raw_loop` → Shows zerocopy advantage (54x faster!)

### Raw vs ZeroCopy (redb_raw_vs_zerocopy)

Direct comparison of raw redb vs zerocopy wrapper across sizes [10, 100, 500, 1000, 5000].

#### Insert Benchmarks

| Benchmark Name | Description |
|----------------|-------------|
| `redb_raw_insert` | Raw Redb API, single transaction |
| `redb_zerocopy_insert` | ZeroCopy API, explicit transaction with loop |
| `redb_zerocopy_bulk_insert` | ZeroCopy API, `put_many()` in transaction |

#### Read Benchmarks (sizes: 100, 1000, 5000)

| Benchmark Name | Description |
|----------------|-------------|
| `redb_raw_read_per_txn` | Raw Redb, new transaction per get (worst case) |
| `redb_raw_read_single_txn` | Raw Redb, single transaction for all reads |
| `redb_zerocopy_read` | ZeroCopy API, explicit read transaction |

**Key Comparisons:**
- `redb_zerocopy_insert` vs `redb_raw_insert` → Pure zerocopy overhead
- `redb_raw_read_per_txn` vs `redb_raw_read_single_txn` → Transaction reuse importance
- `redb_zerocopy_read` vs `redb_raw_read_single_txn` → Wrapper deserialization cost

## How to Compare Benchmarks

### To measure wrapper overhead:
```
overhead = (wrapper_time - raw_time) / raw_time * 100
```

Example:
- `redb_wrapper_bulk` (3.10ms) vs `redb_raw_txn` (1.42ms)
- Overhead = (3.10 - 1.42) / 1.42 * 100 = **118%**

### To measure bulk method speedup:
```
speedup = loop_time / bulk_time
```

Example:
- `redb_wrapper_loop` (27.3ms) vs `redb_wrapper_bulk` (3.10ms)
- Speedup = 27.3 / 3.10 = **8.8x faster**

### To measure transaction importance:
Compare same implementation with different transaction patterns:
- `redb_raw_read_per_txn` vs `redb_raw_read_single_txn`
- Shows cost of creating N transactions vs 1 transaction

## Naming Examples Explained

### `redb_wrapper_bulk`
- **redb**: Uses Redb database
- **wrapper**: Type-safe wrapper API (standard)
- **bulk**: Uses `put_many()` / `get_many()` methods

### `redb_zerocopy_loop`
- **redb**: Uses Redb database
- **zerocopy**: Explicit transaction management
- **loop**: Iterates with per-item operations within transaction

### `sled_raw_batch`
- **sled**: Uses Sled database
- **raw**: Direct Sled API
- **batch**: Uses Sled's native `Batch` API

## Benchmark Groups

### 1. `cross_store_insert`
Tests insert performance across all backends and API levels with varying sizes.

### 2. `cross_store_get`
Tests read performance across all backends and API levels with fixed size (1000).

### 3. `cross_store_bulk_ops`
Tests bulk operation performance for inserts with fixed size (1000).

### 4. `cross_store_secondary_query`
Tests secondary key query performance across implementations with fixed query count (10).

### 5. `redb_raw_vs_zerocopy`
Focused comparison of raw redb vs zerocopy wrapper to measure pure wrapper overhead.

## Visual Output

The naming convention aligns with visualization output:

### Chart File Names
- `insert_comparison_bars.png` - Multi-panel bar charts by size
- `overhead_percentages.png` - Wrapper overhead vs raw
- `bulk_api_speedup.png` - Speedup factors for bulk methods
- `raw_vs_zerocopy_comparison.png` - Line plots comparing raw vs zerocopy

### Flamegraph Paths
Profiling data follows the same pattern:
```
target/criterion/{benchmark_group}/{benchmark_name}/profile/flamegraph.svg
```

Example:
```
target/criterion/cross_store_insert/redb_wrapper_bulk/100/profile/flamegraph.svg
```

## Usage Tips

### Finding Specific Benchmarks

```bash
# List all insert benchmarks
find target/criterion/cross_store_insert -name "flamegraph.svg"

# Compare wrapper overhead for bulk operations
# Look at: redb_wrapper_bulk vs redb_raw_txn

# Find transaction impact
# Compare: redb_wrapper_loop vs redb_wrapper_bulk

# Analyze zerocopy advantage
# Compare: redb_zerocopy_txn (secondary queries) vs redb_raw_loop
```

### Running Specific Benchmarks

```bash
# Run only insert benchmarks
cargo bench --bench cross_store_comparison -- cross_store_insert

# Run only redb benchmarks (using regex)
cargo bench --bench cross_store_comparison -- redb_

# Run only bulk method benchmarks
cargo bench --bench cross_store_comparison -- _bulk
```

## Interpreting Results

### Transaction Overhead Pattern
- `_loop` with wrapper → High overhead (N transactions)
- `_bulk` → Medium overhead (1 transaction)
- `_zerocopy_` → Lower overhead (explicit control)

### Expected Performance Hierarchy

**For Inserts (1000 items):**
1. `redb_raw_txn` (1.42ms) - Fastest (baseline)
2. `redb_wrapper_bulk` (3.10ms) - Good (118% overhead)
3. `redb_zerocopy_loop` (4.34ms) - Acceptable (206% overhead)
4. `redb_wrapper_loop` (27.3ms) - Slowest (1822% overhead)

**For Reads (1000 items):**
1. `redb_raw` (164µs) - Fastest (baseline)
2. `redb_wrapper_bulk` (382µs) - Good (133% overhead)
3. `redb_zerocopy_loop` (692µs) - Acceptable (322% overhead)
4. `redb_wrapper_loop` (895µs) - Slowest (446% overhead)

**For Secondary Queries (10 queries):**
1. **`redb_zerocopy_txn` (5.41µs) - Fastest! (98% faster than raw!)**
2. `redb_raw_loop` (291µs) - Baseline
3. `redb_wrapper_bulk` (470µs) - Good (61% overhead)
4. `redb_wrapper_loop` (1.02ms) - Slowest (248% overhead)

## Naming Convention Benefits

1. **Consistency**: Easy to predict benchmark names
2. **Searchability**: Grep/find by backend or API level
3. **Comparison**: Names clearly indicate what to compare
4. **Documentation**: Self-documenting names
5. **Visualization**: Names work well in charts and tables
