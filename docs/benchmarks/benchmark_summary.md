# Benchmark Results Summary

All times are mean values from Criterion benchmarks.


## Insert Performance

| Implementation | 100 items | 1000 items |
|----------------|-----------|------------|
| 100 | 0.000 ms | 0.000 ms |
| 1000 | 0.000 ms | 0.000 ms |
| raw_redb | 0.374 ms | 1.336 ms |
| raw_sled | 0.707 ms | 4.814 ms |
| report | 0.000 ms | 0.000 ms |
| wrapper_redb_bulk | 0.481 ms | 2.811 ms |
| wrapper_redb_loop | 2.316 ms | 25.057 ms |
| wrapper_sled | 1.957 ms | 15.417 ms |
| zerocopy_redb | 0.619 ms | 3.839 ms |
| zerocopy_redb_bulk | 0.445 ms | 2.770 ms |

## Get Performance (1000 items)

| Implementation | Time |
|----------------|------|
| raw_redb | 154.31 µs |
| raw_sled | 264.40 µs |
| wrapper_redb_bulk | 356.51 µs |
| wrapper_redb_loop | 826.25 µs |
| wrapper_sled | 305.17 µs |
| zerocopy_redb | 614.38 µs |

## Secondary Key Query Performance (10 queries)

| Implementation | Time |
|----------------|------|
| raw_redb_loop | 273.01 µs |
| raw_sled_loop | 614.62 µs |
| wrapper_redb_bulk | 428.31 µs |
| wrapper_redb_loop | 941.50 µs |
| wrapper_sled_loop | 637.40 µs |
| zerocopy_redb_txn | 5.00 µs |

## Bulk Operations (1000 items)

| Implementation | Time |
|----------------|------|
| raw_redb_txn | 1.329 ms |
| raw_sled_batch | 4.653 ms |
| wrapper_redb_loop | 24.736 ms |
| wrapper_sled_loop | 15.652 ms |
| zerocopy_redb_bulk | 2.775 ms |
| zerocopy_redb_txn | 3.799 ms |