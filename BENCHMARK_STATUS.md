# Benchmark Status and Recommendations

## Current Status (After Audit)

### Existing Benchmarks

The benchmark suite currently has 4 files:

1. **cross_store_comparison.rs** (1105 lines) - Main comprehensive benchmarks
2. **redb_wrapper_overhead.rs** (359 lines) - Redb-specific wrapper vs raw
3. **redb_zerocopy_overhead.rs** (343 lines) - Redb-specific zerocopy vs raw
4. **sled_wrapper_overhead.rs** (218 lines) - Sled-specific wrapper vs raw

### Coverage Matrix

| Store | API | Insert (Loop) | Insert (Batch/Txn) | Get | Bulk Ops | Secondary Query |
|-------|-----|---------------|-------------------|-----|----------|-----------------|
| Sled | Raw | ✅ sled_raw_loop | ❌ Missing | ✅ | ✅ | ✅ |
| Sled | Wrapper | ✅ sled_wrapper_loop | ✅ sled_wrapper_txn | ✅ | ✅ | ✅ |
| Redb | Raw | ✅ redb_raw_txn | ✅ redb_raw_txn | ✅ | ✅ | ✅ |
| Redb | Wrapper | ✅ redb_wrapper_loop | ✅ redb_wrapper_bulk | ✅ | ✅ | ✅ |
| Redb | Zerocopy | ✅ redb_zerocopy_loop | ✅ redb_zerocopy_bulk | ✅ | ✅ | ✅ |

### Naming Inconsistencies

Current naming uses mixed conventions:
- `loop` vs `txn` vs `bulk` (should be: `per_item`, `transaction`, `batch`)
- Some use `_insert`, `_read` suffixes, others don't
- Inconsistent grouping (by store vs by operation)

## Recommendations

### 1. Naming Standard

Adopt this naming pattern: `{store}_{api}_{mode}[_{operation}]`

**Store**: `sled`, `redb`
**API**: `raw`, `wrapper`, `zerocopy`
**Mode**:
- `per_item` - Individual operations (N transactions for wrappers)
- `batch` - Bulk operations (`put_many`, etc.)
- `transaction` - Explicit transaction (1 transaction, loop inside)

**Operation** (optional suffix): `insert`, `get`, `query`, etc.

Examples:
```
sled_raw_per_item_insert
sled_raw_transaction_insert
sled_wrapper_per_item_insert
sled_wrapper_batch_insert
redb_raw_transaction_insert  (redb always uses transactions)
redb_wrapper_per_item_insert  (N transactions)
redb_wrapper_batch_insert     (1 transaction via put_many)
redb_zerocopy_transaction_insert  (1 transaction, loop)
redb_zerocopy_batch_insert        (1 transaction, put_many)
```

### 2. Missing Benchmarks

Add these to complete the matrix:

```rust
// In cross_store_comparison.rs, bench_cross_store_insert function:

// Missing: Sled raw with batched insert using sled's batch API
group.bench_with_input(BenchmarkId::new("sled_raw_batch_insert", size), ...);
```

### 3. Benchmark Organization

Keep `cross_store_comparison.rs` as the main file with clear sections:

```rust
// ============================================================================
// SECTION 1: INSERT BENCHMARKS
// ============================================================================
fn bench_cross_store_insert(c: &mut Criterion) {
    // Subsection: Raw APIs
    // - sled_raw_per_item
    // - sled_raw_batch
    // - redb_raw_transaction

    // Subsection: Wrapper APIs
    // - sled_wrapper_per_item
    // - sled_wrapper_transaction
    // - redb_wrapper_per_item
    // - redb_wrapper_batch

    // Subsection: Zerocopy API
    // - redb_zerocopy_transaction
    // - redb_zerocopy_batch
}

// ============================================================================
// SECTION 2: GET BENCHMARKS
// ============================================================================
fn bench_cross_store_get(c: &mut Criterion) {
    // Similar structure...
}
```

### 4. Comparison Tables

Add criterion comparison configuration:

```rust
// At the end of each benchmark group:
group.finish();

// Print comparison table
println!("\n=== Insert Performance Comparison ===");
println!("Baseline (sled_raw_per_item): 100%");
println!("Expected wrapper overhead: 5-15%");
println!("Expected batch speedup: 5-10x");
```

### 5. Documentation

Add doc comments to each benchmark explaining:
- What it measures
- Expected results
- Why this comparison matters

Example:
```rust
/// Measures insert performance across all stores and APIs.
///
/// Comparisons enabled:
/// 1. Raw vs Raw: sled_raw vs redb_raw (baseline store comparison)
/// 2. Raw vs Wrapper: Measures wrapper overhead for each store
/// 3. Per-item vs Batch: Measures transaction batching benefits
/// 4. Wrapper vs Zerocopy: Measures zerocopy API benefits (redb only)
///
/// Expected results:
/// - Wrapper overhead: 5-15%
/// - Batch vs per-item: 5-10x faster
/// - Zerocopy vs wrapper: 10-50% faster for bulk ops
fn bench_cross_store_insert(c: &mut Criterion) {
    ...
}
```

## Implementation Priority

1. ✅ Create benchmark README (DONE)
2. ⏸️ Add missing sled_raw_batch benchmark
3. ⏸️ Rename existing benchmarks for consistency
4. ⏸️ Add section comments and documentation
5. ⏸️ Add comparison summaries

## Verification

Run benchmarks with:
```bash
# All benchmarks
cargo bench --features native

# Specific comparison
cargo bench --features native -- sled_raw

# With flamegraphs
cargo bench --features native --bench cross_store_comparison
```

Check output in `target/criterion/report/index.html`
