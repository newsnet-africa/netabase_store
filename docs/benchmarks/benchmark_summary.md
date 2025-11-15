# Benchmark Results Summary

All times are mean values from Criterion benchmarks.


## Insert Performance

| Implementation | 100 items | 1000 items |
|----------------|-----------|------------|
| 10 | 0.000 ms | 0.000 ms |
| 100 | 0.000 ms | 0.000 ms |
| 1000 | 0.000 ms | 0.000 ms |
| 500 | 0.000 ms | 0.000 ms |
| 5000 | 0.000 ms | 0.000 ms |
| redb_raw_txn | 0.389 ms | 1.374 ms |
| redb_wrapper_bulk | 0.507 ms | 2.924 ms |
| redb_wrapper_loop | 2.565 ms | 25.737 ms |
| redb_zerocopy_bulk | 0.459 ms | 2.827 ms |
| redb_zerocopy_loop | 0.645 ms | 3.958 ms |
| report | 0.000 ms | 0.000 ms |
| sled_raw_batch | 0.714 ms | 5.090 ms |
| sled_raw_loop | 0.718 ms | 4.816 ms |
| sled_wrapper_batch | 0.699 ms | 5.019 ms |
| sled_wrapper_loop | 2.012 ms | 15.944 ms |
| sled_wrapper_txn | 0.759 ms | 5.531 ms |

## Get Performance (1000 items)

| Implementation | Time |
|----------------|------|
| redb_raw | 151.21 µs |
| redb_wrapper_bulk | 519.35 µs |
| redb_wrapper_loop | 858.49 µs |
| redb_zerocopy_loop | 643.75 µs |
| sled_raw | 272.08 µs |
| sled_wrapper_loop | 295.86 µs |
| sled_wrapper_txn | 418.05 µs |

## Secondary Key Query Performance (10 queries)

| Implementation | Time |
|----------------|------|
| redb_raw_loop | 378.59 µs |
| redb_wrapper_bulk | 452.15 µs |
| redb_wrapper_loop | 1030.03 µs |
| redb_zerocopy_txn | 5.11 µs |
| sled_raw_loop | 643.59 µs |
| sled_wrapper_loop | 651.35 µs |
| sled_wrapper_txn | 508.10 µs |

## Bulk Operations (1000 items)

| Implementation | Time |
|----------------|------|
| redb_raw_txn | 2.464 ms |
| redb_wrapper_loop | 34.156 ms |
| redb_zerocopy_bulk | 2.940 ms |
| redb_zerocopy_txn | 5.393 ms |
| sled_raw_batch | 4.827 ms |
| sled_wrapper_loop | 22.047 ms |
| sled_wrapper_txn | 6.197 ms |

## Raw Redb vs ZeroCopy Overhead
