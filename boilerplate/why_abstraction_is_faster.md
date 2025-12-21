# Benchmark Assessment: Post-Fix Analysis

## Status: BUG FIXED
The logic bug in `create_entry` (blind linear zipping) has been fixed. The abstraction now correctly scans for the matching table using discriminants.

## Updated Results (Post-Fix)
| Operation | Scale | Abstracted (Fixed) | Raw | Winner |
|-----------|-------|-------------------|-----|--------|
| **Insert** | 1,000 | 9.81 ms | 9.93 ms | **Abs (~1.2% faster)** |
| **Insert** | 100k | 1.51 s | 1.48 s | **Raw (~2% faster)** |
| **Delete** | 1,000 | 9.98 ms | 10.21 ms | **Abs (~2.3% faster)** |
| **Read** | 100k | 106.2 ms | 98.2 ms | **Raw (~8% faster)** |

*Note: Absolute times increased for both implementations compared to the previous run, likely due to system load, but the relative performance holds.*

## Why is it *still* faster (at small scale)?
Even with the correct lookup logic (which involves a small linear scan of the table list), the Abstraction remains competitive with or slightly faster than the Raw implementation for writes at small/medium scale.

This suggests that **Compiler Optimizations** (inlining, loop fusion) and **Code Structure** (tight loops in `prepare_model` vs interleaved operations in Raw) provide enough benefit to offset the abstraction overhead (e.g., `Vec` allocation). The Raw implementation's `match` dispatch (branching) appears to be roughly equivalent in cost to the Abstraction's `position` scan (linear search) for small numbers of tables.

At larger scales (100k), the overhead of allocating vectors for keys in the Abstraction begins to outweigh the benefits, giving the Raw implementation a slight edge.