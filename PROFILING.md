# Performance Profiling Guide

This guide explains how to profile the netabase_store benchmarks to identify performance bottlenecks.

## 🚨 CRITICAL: Must Use --profile-time Flag

**Flamegraphs will NOT be generated unless you use the `--profile-time` flag:**

```bash
# ❌ WRONG - No flamegraphs
cargo bench

# ✅ CORRECT - Generates flamegraphs
cargo bench -- --profile-time=5
```

**Why?** The profiler needs continuous execution (not short iterations) to collect meaningful CPU samples.

## Prerequisites

The benchmarks are configured with `pprof` integration which automatically generates flamegraphs during benchmark execution.

**Linux users:** Install `perf` for better profiling:
```bash
sudo apt-get install linux-tools-common linux-tools-generic linux-tools-$(uname -r)
```

## Running Benchmarks with Profiling

### Profile All Benchmarks

Run all benchmarks and generate flamegraphs:
```bash
# Quick profiling (5 seconds per benchmark)
cargo bench -- --profile-time=5

# More accurate profiling (10 seconds per benchmark)
cargo bench -- --profile-time=10
```

**Important:** You MUST use the `--profile-time` flag to generate flamegraphs. Regular `cargo bench` runs benchmarks but doesn't generate profiling data.

This will:
- Run all benchmarks with profiling enabled
- Generate HTML reports in `target/criterion/`
- Create flamegraph SVG files in `target/criterion/<benchmark>/profile/flamegraph.svg`

### Profile Specific Benchmark

To profile just one benchmark suite:
```bash
# Profile sled operations
cargo bench --bench sled_wrapper_overhead -- --profile-time=5

# Profile redb operations
cargo bench --bench redb_wrapper_overhead -- --profile-time=5
```

### Profile Specific Test

To profile a specific test within a benchmark:
```bash
# Profile only the insert operations
cargo bench --bench sled_wrapper_overhead -- --profile-time=5 insert

# Profile only get operations
cargo bench --bench redb_wrapper_overhead -- --profile-time=5 get
```

## Understanding the Output

### Criterion Reports

HTML reports are located in `target/criterion/<benchmark_name>/report/index.html`

Open these in a browser to see:
- Execution time graphs
- Statistical analysis
- Comparison with previous runs

### Flamegraphs

Flamegraphs are SVG files located in `target/criterion/<benchmark_name>/profile/flamegraph.svg`

**How to read flamegraphs:**
- **Width** = time spent in function (wider = more time)
- **Height** = call stack depth
- **Color** = random (just for visual separation)
- **Hover** = shows function name and percentage

**What to look for:**
- Wide plateaus = bottlenecks
- Functions at the bottom = leaf functions (actual work)
- Functions at the top = entry points

### Example: Finding Bottlenecks

1. Open `target/criterion/insert/wrapper/100/profile/flamegraph.svg`
2. Look for the widest sections
3. Common bottlenecks:
   - `bincode::encode` - serialization overhead
   - `sled::Tree::insert` - database write operations
   - Memory allocation functions
   - Lock contention (`parking_lot`, `std::sync`)

## Comparing Performance

### Compare Wrapper vs Raw Implementation

The benchmarks include both raw database operations and wrapped implementations:

```bash
# Run insert benchmarks
cargo bench --bench sled_wrapper_overhead -- insert
```

Compare the flamegraphs:
- `target/criterion/insert/raw_sled/<size>/profile/flamegraph.svg`
- `target/criterion/insert/wrapper/<size>/profile/flamegraph.svg`

This shows the overhead added by the type-safe wrapper layer.

### Compare Different Sizes

Each benchmark runs with different data sizes (100, 1000, 5000 records):

```bash
cargo bench --bench sled_wrapper_overhead
```

Compare flamegraphs across sizes to see how performance characteristics change with scale.

## Advanced Profiling

### Custom Sample Rate

The default sample rate is 100Hz (samples per second). To change it, edit the benchmark files:

```rust
fn configure_criterion() -> Criterion {
    Criterion::default()
        .with_profiler(PProfProfiler::new(1000, Output::Flamegraph(None))) // 1000Hz
}
```

Higher sample rates = more accurate but slower benchmarks.

### Profile in Release Mode

Benchmarks automatically run in release mode with optimizations. To verify:
```bash
cargo bench --verbose
```

### CPU-specific Profiling

On Linux with `perf` installed, you can get more detailed CPU information:

```bash
perf record -F 99 -g -- cargo bench --bench sled_wrapper_overhead -- --test
perf report
```

## Interpreting Results

### Expected Overhead

Type-safe wrapper overhead should be:
- **Insert**: 5-15% slower (serialization + index management)
- **Get**: 10-20% slower (deserialization + type checking)
- **Iteration**: 15-25% slower (per-item deserialization)

### Red Flags

Watch for:
- **Excessive allocation**: Many `alloc::alloc` calls
- **Lock contention**: Time spent in `parking_lot::lock`
- **Unnecessary copies**: Multiple `memcpy` operations
- **Inefficient serialization**: Deep call stacks in `bincode`

## Optimization Tips

Based on profiling results:

1. **Serialization bottleneck** → Consider smaller data types or custom serialization
2. **Index lookup overhead** → Batch operations when possible
3. **Lock contention** → Use concurrent data structures or reduce critical sections
4. **Memory allocation** → Pre-allocate buffers or use object pools

## Continuous Performance Monitoring

Run benchmarks regularly and save results:

```bash
# Save baseline
cargo bench -- --save-baseline before

# After changes, compare
cargo bench -- --baseline before
```

Criterion will show performance deltas in the HTML reports.

## Troubleshooting

### "Permission denied" errors on Linux

If you see permission errors with perf:
```bash
sudo sysctl -w kernel.perf_event_paranoid=1
```

### Flamegraphs not generated

Ensure `pprof` dependencies are installed:
```bash
cargo update -p pprof
cargo clean && cargo bench
```

### High variance in results

- Close other applications
- Disable CPU frequency scaling: `sudo cpupower frequency-set --governor performance`
- Run benchmarks multiple times: `cargo bench -- --sample-size 100`

## Further Reading

- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- [Flamegraph.pl](https://github.com/brendangregg/FlameGraph)
- [pprof Documentation](https://docs.rs/pprof/)
