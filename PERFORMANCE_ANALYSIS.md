# Performance Analysis

## Overview
This document analyzes the performance characteristics of the Netabase Store abstraction layer compared to raw redb usage.

## Test Methodology
- **Backend**: redb (in-memory mode to isolate abstraction overhead from I/O)
- **Test Framework**: Criterion benchmarks
- **Comparison**: Abstracted Netabase API vs Raw redb implementation
- **Hardware**: Varies by system (benchmarks run in-memory)

---

## Benchmark Results Summary

### Minimal Configuration (Single Table, No Auxiliary Keys)

This benchmark tests the purest abstraction overhead with a single table containing only a primary key (u64 ID).

#### Insert Performance (10,000 items)

| Implementation | Time (10k ops) | Time/Op | Overhead vs Raw |
|----------------|----------------|---------|-----------------|
| Raw Redb       | 8.28 ms        | 0.82 µs | -               |
| Abstracted (Naive) | 19.35 ms   | 1.93 µs | +135%           |
| **Abstracted (Batch)** | **12.98 ms** | **1.29 µs** | **+57%** |

**Analysis**: 
- **Naive Usage**: Calling `txn.create()` for each item re-opens all tables (main, secondary, etc.) for every operation. This adds ~0.5µs overhead per op.
- **Batch Usage**: Using `txn.prepare_model()` once significantly reduces overhead.
- **Key Optimization**: Implementing `get_primary_key_ref` (avoiding clones) reduced the overhead from +66% to +57% in batch mode. The remaining overhead is likely due to trait dispatch and serialization wrappers.

#### Read Performance (10,000 items)

| Implementation | Time (10k ops) | Time/Op | Overhead vs Raw |
|----------------|----------------|---------|-----------------|
| Raw Redb       | 4.14 ms        | 0.41 µs | -               |
| Abstracted (Naive) | 12.98 ms   | 1.30 µs | +213%           |
| **Abstracted (Batch)** | **8.91 ms** | **0.89 µs** | **+115%** |

**Analysis**:
- Similar to writes, repeatedly opening tables adds significant cost (~0.4µs/op).
- **Batch Optimization**: Pre-opening tables with `prepare_model` improves performance by ~30%.
- **Remaining Overhead**: Read operations also suffer from key cloning (`read(&key)` vs raw `table.get(key_ref)`).

---

### Complex Configuration (Multiple Tables, Auxiliary Keys, Subscriptions)

The full CRUD benchmark tests realistic usage with:
- Multiple models (User, Post)
- 4 auxiliary index keys per model
- 4 subscription topics
- Relational links
- Heavy blob data (~1MB per record)

#### Insert Performance (10,000 items)

| Implementation | Time (10k ops) | Time/Op | Overhead vs Raw |
|----------------|----------------|---------|-----------------|
| Raw Redb       | 1.87 s         | 187 µs  | -               |
| **Abstracted (Batch)** | **1.91 s** | **191 µs** | **+2.1%** |
| Libp2p (Naive) | 4.09 s         | 409 µs  | +118%           |

**Analysis**:
- **Batching Wins**: The Abstracted (Batch) implementation maintains near-parity with Raw implementation (~2% overhead). This confirms that the abstraction cost is negligible compared to the cost of maintaining complex indexes and writing data.
- **Libp2p Overhead**: The `Libp2pRedbStore` implementation continues to show ~2x overhead due to the transaction-per-record requirement of the trait.

### Content-Addressed Models (Immutable Data)

New benchmark testing the performance of `#[netabase_content_addressed]` models. These models use their hash as the primary key and are immutable.

#### Insert Performance (10,000 items)

| Implementation | Time (10k ops) | Time/Op | Notes |
|----------------|----------------|---------|-------|
| **Content-Addressed** | **142 ms** | **14.2 µs** | **Simulating P2P Sync (Pre-hashed)** |

**Analysis**:
- **High Throughput**: Inserting 10,000 content-addressed items takes only ~142ms.
- **Scenario**: This benchmark simulates a P2P sync scenario where we receive `Envelopes` (data + hash) from a peer. The hashing cost is already paid.
- **Efficiency**: The low cost (14.2µs vs 191µs for User) reflects the simpler structure (fewer indexes, no blobs) and efficient `Envelope` storage.

---

## Bottleneck Analysis

### 1. Repeated Table Opening (Solved)
**Issue**: The convenience methods `txn.create()`, `txn.read()`, etc., open the underlying redb tables on every call.
**Impact**: ~0.5µs per operation. Significant for small records/batches.
**Solution**: Use `txn.prepare_model::<M>()` for batch operations.
```rust
// BAD (Naive)
for item in items {
    txn.create(item)?; // Opens tables 10,000 times
}

// GOOD (Batch)
let mut tables = txn.prepare_model::<Item>()?; // Opens tables ONCE
for item in items {
    item.create_entry(&mut tables)?;
}
```

### 2. Primary Key Cloning (Partially Solved)
**Issue**: `NetabaseModel::get_primary_key()` previously returned `Self::Primary` by value. For types like `String`, this forced an allocation on every database access.
**Optimization**: Introduced `get_primary_key_ref()` to return borrowed keys.
**Impact**: Reduced batch insert overhead from +66% to +57%.
**Remaining**: Some paths may still require owned keys due to `redb` trait bounds or complex key types.

### 3. Libp2p Trait Constraints
**Issue**: The `libp2p::kad::store::RecordStore` trait methods (`put`, `get`) operate on single records. This forces the implementation to open a transaction for every operation.
**Impact**: ~100% overhead (2x execution time) compared to batched operations.
**Mitigation**: This is an inherent trade-off for compatibility with the `libp2p` ecosystem. For high-throughput scenarios, custom batching APIs should be preferred over the standard `RecordStore` trait.

### 4. Serialization
**Status**: Efficient. Both Raw and Abstracted implementations use `postcard` (or `bincode`) effectively. The serialization overhead is matched.

---

## Recommendations

### For Developers
1.  **Always use `prepare_model` for loops**: If you are processing more than a few items, explicit table preparation is mandatory for performance.
2.  **Accept the trade-off**: The overhead (0.5µs/op) provides type safety, automatic indexing, and relationships. For complex models, this cost is invisible.

### For Library Maintainers
1.  **Optimize Key Access**: Continue to refine `NetabaseModel` to allow borrowing keys where possible.
2.  **Cache Tables**: Investigate caching open tables within the `NetabaseRedbTransaction` struct to make the convenience methods (`create`, `read`) faster automatically, though this may have borrowing complexity.

## Conclusion

The "extremely high overhead" (150%+) observed in initial benchmarks was primarily due to **API usage patterns** (repeatedly opening tables). With proper batching patterns (`prepare_model`), the overhead drops significantly. The remaining overhead is due to safety/ergonomic choices (owned keys) which have negligible impact on real-world workloads involving indexes or larger data.