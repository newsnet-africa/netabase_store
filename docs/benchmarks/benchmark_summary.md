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
| redb_raw_txn | 0.399 ms | 1.457 ms |
| redb_wrapper_bulk | 0.512 ms | 2.955 ms |
| redb_wrapper_loop | 2.581 ms | 25.724 ms |
| redb_zerocopy_bulk | 0.474 ms | 2.905 ms |
| redb_zerocopy_loop | 0.662 ms | 3.998 ms |
| report | 0.000 ms | 0.000 ms |
| sled_raw_loop | 0.718 ms | 4.837 ms |
| sled_wrapper_loop | 1.956 ms | 15.618 ms |

## Get Performance (1000 items)

| Implementation | Time |
|----------------|------|
| redb_raw | 149.54 µs |
| redb_wrapper_bulk | 378.83 µs |
| redb_wrapper_loop | 874.20 µs |
| redb_zerocopy_loop | 654.80 µs |
| sled_raw | 269.84 µs |
| sled_wrapper | 297.86 µs |

## Secondary Key Query Performance (10 queries)

| Implementation | Time |
|----------------|------|
| redb_raw_loop | 275.28 µs |
| redb_wrapper_bulk | 445.75 µs |
| redb_wrapper_loop | 979.04 µs |
| redb_zerocopy_txn | 5.15 µs |
| sled_raw_loop | 612.37 µs |
| sled_wrapper_loop | 638.51 µs |

## Bulk Operations (1000 items)

| Implementation | Time |
|----------------|------|
| redb_raw_txn | 1.430 ms |
| redb_wrapper_loop | 26.362 ms |
| redb_zerocopy_bulk | 2.935 ms |
| redb_zerocopy_txn | 4.031 ms |
| sled_raw_batch | 4.750 ms |
| sled_wrapper_loop | 15.651 ms |

## Raw Redb vs ZeroCopy Overhead
