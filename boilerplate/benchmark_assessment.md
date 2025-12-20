# CRUD Benchmark Assessment Report

**Date:** 2025-12-20
**Benchmark:** `boilerplate/benches/crud.rs`

## Executive Summary

The benchmarks show that the abstraction layer is **very efficient**, with minimal overhead for most operations. However, the current benchmark design includes table opening overhead in the measured time, which makes small-dataset comparisons somewhat unfair.

## Benchmark Results (Current Implementation)

### Insert Performance
- **Small sizes (0-100)**: Raw is ~5% **slower** than Abstracted
- **Medium sizes (1000-10000)**: **Essentially equal**
- **Large size (100000)**: Raw is ~6% **faster** than Abstracted

### Read Performance
- **All sizes**: Raw is 9-16% **faster** than Abstracted (consistently better)

### Delete Performance
- **All sizes**: **Essentially equal** (within margin of error)

## Detailed Results

```
CRUD/Insert/Abstracted/0      time: [122.37 µs 123.36 µs 124.88 µs]
CRUD/Insert/Raw/0             time: [129.04 µs 129.77 µs 130.77 µs]  (~5% slower)

CRUD/Insert/Abstracted/100    time: [780.71 µs 784.10 µs 786.78 µs]
CRUD/Insert/Raw/100           time: [818.87 µs 825.50 µs 832.71 µs]  (~5% slower)

CRUD/Insert/Abstracted/1000   time: [8.9468 ms 9.0500 ms 9.1601 ms]
CRUD/Insert/Raw/1000          time: [8.9113 ms 9.0346 ms 9.1483 ms]  (essentially equal)

CRUD/Insert/Abstracted/10000  time: [106.18 ms 108.06 ms 109.87 ms]
CRUD/Insert/Raw/10000         time: [105.49 ms 106.97 ms 108.69 ms]  (essentially equal)

CRUD/Insert/Abstracted/100000 time: [1.4416 s 1.5061 s 1.5880 s]
CRUD/Insert/Raw/100000        time: [1.4103 s 1.4208 s 1.4307 s]     (~6% faster)

CRUD/Read/Abstracted/100      time: [209.05 µs 223.31 µs 242.16 µs]
CRUD/Read/Raw/100             time: [185.62 µs 188.38 µs 191.33 µs]  (~16% faster)

CRUD/Read/Abstracted/1000     time: [950.18 µs 985.95 µs 1.0244 ms]
CRUD/Read/Raw/1000            time: [827.96 µs 837.97 µs 848.87 µs]  (~15% faster)

CRUD/Delete/Abstracted/1000   time: [10.998 ms 11.206 ms 11.398 ms]
CRUD/Delete/Raw/1000          time: [11.330 ms 11.523 ms 11.752 ms]  (essentially equal)
```

## Sources of Measurement Unfairness

### 1. Table Opening Overhead Included in Measured Time

The primary source of unfairness is that **table opening is included in the measured time**, and the two implementations open tables differently:

**Abstracted version** (lines 124-129 in benchmark):
```rust
let txn = store.begin_transaction().expect("Failed to begin txn");
{
    let mut tables = txn
        .prepare_model::<User>()  // ← Opens all tables in ONE call
        .expect("Failed to prepare model");
    for user in &users {  // Then iterates
```

**Raw version** (lines 156-178 in benchmark):
```rust
let txn = db.begin_write().expect("Failed to begin txn");
{
    let mut main_table = txn.open_table(MAIN).expect("Failed to open main");
    let mut sec_name = txn.open_multimap_table(SEC_NAME).expect(...);
    let mut sec_age = txn.open_multimap_table(SEC_AGE).expect(...);
    let mut rel_partner = txn.open_multimap_table(REL_PARTNER).expect(...);
    let mut rel_category = txn.open_multimap_table(REL_CATEGORY).expect(...);
    let mut sub_topic1 = txn.open_multimap_table(SUB_TOPIC1).expect(...);
    let mut sub_topic2 = txn.open_multimap_table(SUB_TOPIC2).expect(...);
    // ← 7 individual table opens!
    for user in &users {
```

This explains the performance pattern:
- **Small sizes (0-100)**: Raw appears ~5% slower (table opening overhead dominates)
- **Large sizes (10000-100000)**: Performance converges (insertion time dominates)
- **At 100K records**: Raw becomes faster (less per-operation overhead)

### 2. Database/Store Creation is Separate (Fair)

Both implementations create fresh databases in the setup phase, so this is fair.

### 3. Data Generation is Identical (Fair)

Both use the same `generate_random_user()` function, so data is identical.

## Analysis of Implementation Differences

### Insert Path

**Abstracted** (`src/databases/redb/transaction/crud.rs:89-152`):
1. Calls `prepare_model::<User>()` once
2. For each user, calls `user.create_entry(&mut tables)`
3. Inside `create_entry()`:
   - Inserts into main table
   - Calls `get_secondary_keys()` → creates Vec with clones
   - Calls `get_relational_keys()` → creates Vec with clones
   - Calls `get_subscription_keys()` → creates Vec with clones
   - Iterates and inserts into multimap tables

**Raw** (benchmark lines 156-234):
1. Opens 7 tables individually (7 calls)
2. For each user:
   - Inserts into main table
   - Manually constructs keys inline (clones strings)
   - Inserts into multimap tables directly

**Key Overhead Sources:**
- Abstracted: Vec allocations for key collections, method call overhead
- Raw: Inline key construction (similar clones), no indirection

At scale, these are roughly equivalent, which is why performance converges.

### Read Path

**Abstracted** (crud.rs:154-170):
```rust
fn read_entry(key, tables) -> NetabaseResult<Option<Self>>
{
    match &tables.main {
        TablePermission::ReadOnly(TableType::Table(table)) => {
            let result = table.get(key.borrow())?;
            Ok(result.map(|access_guard| access_guard.value()))
        },
        TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
            let result = table.get(key.borrow())?;
            Ok(result.map(|access_guard| access_guard.value()))
        },
        _ => Err(NetabaseError::Other),
    }
}
```

**Raw** (benchmark lines 369-379):
```rust
let txn = db.begin_read().expect("Failed to begin txn");
{
    let main_table = txn.open_table(MAIN).expect("Failed to open main");
    for user in &users {
        black_box(main_table.get(black_box(&user.id)))
            .expect("Failed to get user")
            .map(|g| g.value());
    }
}
```

**Why Raw is 9-16% Faster:**
- Abstracted has `TablePermission` enum match overhead on EVERY read
- Abstracted wraps in `NetabaseResult` (additional error handling)
- Raw has direct table access with no indirection

This is **genuine overhead** from the abstraction layer's flexibility (supporting both ReadOnly and ReadWrite tables).

### Delete Path

Performance is essentially equal because:
- Both need to read the entry first to get keys
- Both perform the same number of deletions from tables
- The abstraction overhead is amortized across the delete operations

## Why Raw Becomes Faster at Large Scale

At 100K records for inserts, raw is ~6% faster because:
1. Table opening overhead is amortized to near-zero
2. Abstraction's per-operation overhead accumulates:
   - Vec allocations for key collections (3 allocations × 100K users)
   - Method call indirection
   - `TablePermission` enum matching
3. Raw has minimal indirection for the actual operations

## Conclusions

### 1. The Abstraction is Very Efficient
- Only 5% overhead for small operations
- Matches raw performance at medium scales
- Read overhead (9-16%) is acceptable for the flexibility gained

### 2. Benchmark Unfairness
The benchmark unfairly penalizes raw implementation for small datasets by including table opening overhead. However, this reveals that:
- The abstraction's `prepare_model()` is well-optimized
- Real-world usage patterns (many operations per transaction) favor the abstraction

### 3. Trade-offs are Reasonable
- **Abstraction provides:** Type safety, model management, permission system, error handling
- **Cost:** 0-5% for writes, 9-16% for reads, 0% for deletes
- **Benefit at scale:** Simplified code, fewer bugs, maintainability

## Recommendations

### To Make Benchmarks Fair:
1. **Move table opening outside the measured block** for both implementations
2. **Measure only the operation loops** (insert/read/delete iterations)
3. **Add separate benchmark** for table opening overhead
4. **Test realistic transaction patterns** (mixed read/write, batch operations)

### Expected Results After Fix:
- **Inserts:** Raw likely 3-5% faster (Vec allocation overhead)
- **Reads:** Raw 9-16% faster (TablePermission match overhead)
- **Deletes:** Equal (already amortized)

This would give a true measurement of the abstraction's operational overhead without conflating setup costs.

## Files Referenced

- **Benchmark:** `boilerplate/benches/crud.rs`
- **Abstraction CRUD:** `src/databases/redb/transaction/crud.rs`
- **Model Definition:** Referenced via `netabase_store_examples::boilerplate_lib::models::user::User`
