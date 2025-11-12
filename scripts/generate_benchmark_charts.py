#!/usr/bin/env python3
"""
Generate comparative benchmark charts from Criterion output.

This script parses the JSON data from Criterion benchmarks and generates
additional comparative visualizations beyond the default violin plots and line graphs.
"""

import json
import os
from pathlib import Path
from typing import Dict, List, Tuple
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import numpy as np

# Configuration
CRITERION_DIR = Path("target/criterion")
OUTPUT_DIR = Path("docs/benchmarks")
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

# Color scheme - using consistent naming
COLORS = {
    # Sled benchmarks
    'sled_raw': '#1f77b4',
    'sled_raw_loop': '#1f77b4',
    'sled_raw_batch': '#1a5a8a',
    'sled_wrapper': '#ff7f0e',
    'sled_wrapper_loop': '#ff7f0e',

    # Redb raw benchmarks
    'redb_raw': '#2ca02c',
    'redb_raw_txn': '#2ca02c',
    'redb_raw_loop': '#2ca02c',
    'redb_raw_insert': '#2ca02c',
    'redb_raw_read_per_txn': '#267326',
    'redb_raw_read_single_txn': '#2ca02c',

    # Redb wrapper benchmarks
    'redb_wrapper_loop': '#d62728',
    'redb_wrapper_bulk': '#9467bd',

    # Redb zerocopy benchmarks
    'redb_zerocopy_loop': '#8c564b',
    'redb_zerocopy_bulk': '#e377c2',
    'redb_zerocopy_txn': '#7f7f7f',
    'redb_zerocopy_insert': '#8c564b',
    'redb_zerocopy_bulk_insert': '#e377c2',
    'redb_zerocopy_read': '#7f7f7f',
}

def load_estimates(benchmark_path: Path) -> Dict:
    """Load estimates.json from a benchmark directory."""
    estimates_file = benchmark_path / "new" / "estimates.json"
    if not estimates_file.exists():
        return None

    with open(estimates_file, 'r') as f:
        return json.load(f)

def extract_mean_time_ns(estimates: Dict) -> float:
    """Extract mean time in nanoseconds from estimates."""
    if not estimates:
        return None
    return estimates.get('mean', {}).get('point_estimate', 0)

def parse_cross_store_benchmarks():
    """Parse all cross-store benchmark results."""
    benchmarks = {
        'insert': {},
        'get': {},
        'bulk': {},
        'secondary_query': {}
    }

    # Parse insert benchmarks (have size parameter)
    insert_dir = CRITERION_DIR / "cross_store_insert"
    if insert_dir.exists():
        for impl_dir in insert_dir.iterdir():
            if not impl_dir.is_dir():
                continue
            impl_name = impl_dir.name
            if impl_name not in benchmarks['insert']:
                benchmarks['insert'][impl_name] = {}

            for size_dir in impl_dir.iterdir():
                if not size_dir.is_dir():
                    continue
                try:
                    size = int(size_dir.name)
                    estimates = load_estimates(size_dir)
                    if estimates:
                        time_ns = extract_mean_time_ns(estimates)
                        benchmarks['insert'][impl_name][size] = time_ns
                except ValueError:
                    continue

    # Parse get benchmarks (no size parameter)
    get_dir = CRITERION_DIR / "cross_store_get"
    if get_dir.exists():
        for impl_dir in get_dir.iterdir():
            if not impl_dir.is_dir() or impl_dir.name in ['report']:
                continue
            impl_name = impl_dir.name
            estimates = load_estimates(impl_dir)
            if estimates:
                benchmarks['get'][impl_name] = extract_mean_time_ns(estimates)

    # Parse bulk benchmarks
    bulk_dir = CRITERION_DIR / "cross_store_bulk"
    if bulk_dir.exists():
        for impl_dir in bulk_dir.iterdir():
            if not impl_dir.is_dir() or impl_dir.name in ['report']:
                continue
            impl_name = impl_dir.name
            estimates = load_estimates(impl_dir)
            if estimates:
                benchmarks['bulk'][impl_name] = extract_mean_time_ns(estimates)

    # Parse secondary query benchmarks
    sec_dir = CRITERION_DIR / "cross_store_secondary_query"
    if sec_dir.exists():
        for impl_dir in sec_dir.iterdir():
            if not impl_dir.is_dir() or impl_dir.name in ['report']:
                continue
            impl_name = impl_dir.name
            estimates = load_estimates(impl_dir)
            if estimates:
                benchmarks['secondary_query'][impl_name] = extract_mean_time_ns(estimates)

    # Parse raw vs zerocopy benchmarks
    benchmarks['raw_vs_zerocopy'] = {}
    rawvszc_dir = CRITERION_DIR / "redb_raw_vs_zerocopy"
    if rawvszc_dir.exists():
        for impl_dir in rawvszc_dir.iterdir():
            if not impl_dir.is_dir() or impl_dir.name in ['report']:
                continue
            impl_name = impl_dir.name

            # Check if this has size subdirectories
            has_sizes = False
            for entry in impl_dir.iterdir():
                if entry.is_dir() and entry.name.isdigit():
                    has_sizes = True
                    break

            if has_sizes:
                if impl_name not in benchmarks['raw_vs_zerocopy']:
                    benchmarks['raw_vs_zerocopy'][impl_name] = {}

                for size_dir in impl_dir.iterdir():
                    if not size_dir.is_dir():
                        continue
                    try:
                        size = int(size_dir.name)
                        estimates = load_estimates(size_dir)
                        if estimates:
                            time_ns = extract_mean_time_ns(estimates)
                            benchmarks['raw_vs_zerocopy'][impl_name][size] = time_ns
                    except ValueError:
                        continue
            else:
                # No size subdirectories
                estimates = load_estimates(impl_dir)
                if estimates:
                    benchmarks['raw_vs_zerocopy'][impl_name] = extract_mean_time_ns(estimates)

    return benchmarks

def generate_insert_comparison_bar_chart(data: Dict):
    """Generate grouped bar chart comparing insert performance across implementations."""
    # Get all available sizes
    all_sizes = set()
    for impl_data in data.values():
        all_sizes.update(impl_data.keys())
    sizes = sorted([s for s in all_sizes if isinstance(s, int)])

    # Create subplots based on number of sizes
    n_sizes = len(sizes)
    n_cols = min(3, n_sizes)
    n_rows = (n_sizes + n_cols - 1) // n_cols

    fig, axes = plt.subplots(n_rows, n_cols, figsize=(6*n_cols, 5*n_rows))
    fig.suptitle('Insert Performance Comparison', fontsize=16, fontweight='bold')

    if n_sizes == 1:
        axes = [axes]
    elif n_rows == 1:
        axes = axes if n_sizes > 1 else [axes]
    else:
        axes = axes.flatten()

    impl_names = list(data.keys())

    for idx, size in enumerate(sizes):
        if idx >= len(axes):
            break
        ax = axes[idx]

        # Collect times for this size
        impl_times = []
        impl_labels = []
        colors = []

        for impl in impl_names:
            if size in data[impl]:
                time_ms = data[impl][size] / 1_000_000  # Convert ns to ms
                impl_times.append(time_ms)
                impl_labels.append(impl)
                colors.append(COLORS.get(impl, '#999999'))

        # Create bar chart
        x_pos = np.arange(len(impl_labels))
        bars = ax.bar(x_pos, impl_times, color=colors)

        # Customize
        ax.set_xlabel('Implementation', fontsize=12)
        ax.set_ylabel('Time (ms)', fontsize=12)
        ax.set_title(f'Size: {size} items', fontsize=14)
        ax.set_xticks(x_pos)
        ax.set_xticklabels(impl_labels, rotation=45, ha='right')
        ax.grid(axis='y', alpha=0.3)

        # Add value labels on bars
        for bar in bars:
            height = bar.get_height()
            ax.text(bar.get_x() + bar.get_width()/2., height,
                    f'{height:.2f}',
                    ha='center', va='bottom', fontsize=9)

    # Hide unused subplots
    for idx in range(n_sizes, len(axes)):
        axes[idx].set_visible(False)

    plt.tight_layout()
    plt.savefig(OUTPUT_DIR / 'insert_comparison_bars.png', dpi=300, bbox_inches='tight')
    plt.close()

def generate_overhead_percentage_chart(data: Dict):
    """Generate chart showing overhead percentage vs raw implementations."""
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    fig.suptitle('Wrapper Overhead vs Raw Implementations', fontsize=16, fontweight='bold')

    def calc_overhead(wrapper_time, raw_time):
        if raw_time == 0:
            return 0
        return ((wrapper_time - raw_time) / raw_time) * 100

    # Insert overhead - Size 100
    if 100 in data['insert'].get('raw_sled', {}):
        ax = axes[0, 0]
        raw_sled_100 = data['insert']['raw_sled'][100]
        raw_redb_100 = data['insert']['raw_redb'][100]

        overheads = []
        labels = []
        colors_list = []

        if 'wrapper_sled' in data['insert'] and 100 in data['insert']['wrapper_sled']:
            overhead = calc_overhead(data['insert']['wrapper_sled'][100], raw_sled_100)
            overheads.append(overhead)
            labels.append('Sled Wrapper')
            colors_list.append(COLORS['wrapper_sled'])

        if 'wrapper_redb_loop' in data['insert'] and 100 in data['insert']['wrapper_redb_loop']:
            overhead = calc_overhead(data['insert']['wrapper_redb_loop'][100], raw_redb_100)
            overheads.append(overhead)
            labels.append('Redb Wrapper\n(loop)')
            colors_list.append(COLORS['wrapper_redb_loop'])

        if 'wrapper_redb_bulk' in data['insert'] and 100 in data['insert']['wrapper_redb_bulk']:
            overhead = calc_overhead(data['insert']['wrapper_redb_bulk'][100], raw_redb_100)
            overheads.append(overhead)
            labels.append('Redb Wrapper\n(bulk)')
            colors_list.append(COLORS['wrapper_redb_bulk'])

        if 'zerocopy_redb' in data['insert'] and 100 in data['insert']['zerocopy_redb']:
            overhead = calc_overhead(data['insert']['zerocopy_redb'][100], raw_redb_100)
            overheads.append(overhead)
            labels.append('Redb ZeroCopy')
            colors_list.append(COLORS['zerocopy_redb'])

        bars = ax.barh(labels, overheads, color=colors_list)
        ax.set_xlabel('Overhead (%)', fontsize=12)
        ax.set_title('Insert Overhead (100 items)', fontsize=14)
        ax.grid(axis='x', alpha=0.3)
        ax.axvline(x=0, color='black', linestyle='-', linewidth=0.5)

        # Add value labels
        for bar, overhead in zip(bars, overheads):
            width = bar.get_width()
            ax.text(width, bar.get_y() + bar.get_height()/2,
                    f'{overhead:+.1f}%',
                    ha='left' if width >= 0 else 'right',
                    va='center', fontsize=10, fontweight='bold')

    # Insert overhead - Size 1000
    if 1000 in data['insert'].get('raw_sled', {}):
        ax = axes[0, 1]
        raw_sled_1000 = data['insert']['raw_sled'][1000]
        raw_redb_1000 = data['insert']['raw_redb'][1000]

        overheads = []
        labels = []
        colors_list = []

        if 'wrapper_sled' in data['insert'] and 1000 in data['insert']['wrapper_sled']:
            overhead = calc_overhead(data['insert']['wrapper_sled'][1000], raw_sled_1000)
            overheads.append(overhead)
            labels.append('Sled Wrapper')
            colors_list.append(COLORS['wrapper_sled'])

        if 'wrapper_redb_loop' in data['insert'] and 1000 in data['insert']['wrapper_redb_loop']:
            overhead = calc_overhead(data['insert']['wrapper_redb_loop'][1000], raw_redb_1000)
            overheads.append(overhead)
            labels.append('Redb Wrapper\n(loop)')
            colors_list.append(COLORS['wrapper_redb_loop'])

        if 'wrapper_redb_bulk' in data['insert'] and 1000 in data['insert']['wrapper_redb_bulk']:
            overhead = calc_overhead(data['insert']['wrapper_redb_bulk'][1000], raw_redb_1000)
            overheads.append(overhead)
            labels.append('Redb Wrapper\n(bulk)')
            colors_list.append(COLORS['wrapper_redb_bulk'])

        if 'zerocopy_redb_bulk' in data['insert'] and 1000 in data['insert']['zerocopy_redb_bulk']:
            overhead = calc_overhead(data['insert']['zerocopy_redb_bulk'][1000], raw_redb_1000)
            overheads.append(overhead)
            labels.append('Redb ZeroCopy\n(bulk)')
            colors_list.append(COLORS['zerocopy_redb_bulk'])

        bars = ax.barh(labels, overheads, color=colors_list)
        ax.set_xlabel('Overhead (%)', fontsize=12)
        ax.set_title('Insert Overhead (1000 items)', fontsize=14)
        ax.grid(axis='x', alpha=0.3)
        ax.axvline(x=0, color='black', linestyle='-', linewidth=0.5)

        for bar, overhead in zip(bars, overheads):
            width = bar.get_width()
            ax.text(width, bar.get_y() + bar.get_height()/2,
                    f'{overhead:+.1f}%',
                    ha='left' if width >= 0 else 'right',
                    va='center', fontsize=10, fontweight='bold')

    # Get overhead
    ax = axes[1, 0]
    if 'raw_redb' in data['get']:
        raw_redb_get = data['get']['raw_redb']

        overheads = []
        labels = []
        colors_list = []

        for impl in ['wrapper_redb_loop', 'wrapper_redb_bulk', 'zerocopy_redb']:
            if impl in data['get']:
                overhead = calc_overhead(data['get'][impl], raw_redb_get)
                overheads.append(overhead)
                label = impl.replace('wrapper_redb_', 'Wrapper\n(').replace('zerocopy_redb', 'ZeroCopy')
                if 'Wrapper' in label:
                    label += ')'
                labels.append(label)
                colors_list.append(COLORS[impl])

        bars = ax.barh(labels, overheads, color=colors_list)
        ax.set_xlabel('Overhead (%)', fontsize=12)
        ax.set_title('Get Overhead (1000 items)', fontsize=14)
        ax.grid(axis='x', alpha=0.3)
        ax.axvline(x=0, color='black', linestyle='-', linewidth=0.5)

        for bar, overhead in zip(bars, overheads):
            width = bar.get_width()
            ax.text(width, bar.get_y() + bar.get_height()/2,
                    f'{overhead:+.1f}%',
                    ha='left' if width >= 0 else 'right',
                    va='center', fontsize=10, fontweight='bold')

    # Secondary query overhead
    ax = axes[1, 1]
    if 'raw_redb_loop' in data['secondary_query']:
        raw_redb_sec = data['secondary_query']['raw_redb_loop']

        overheads = []
        labels = []
        colors_list = []

        for impl in ['wrapper_redb_loop', 'wrapper_redb_bulk', 'zerocopy_redb_txn']:
            if impl in data['secondary_query']:
                overhead = calc_overhead(data['secondary_query'][impl], raw_redb_sec)
                overheads.append(overhead)
                label = impl.replace('wrapper_redb_', 'Wrapper\n(').replace('zerocopy_redb_txn', 'ZeroCopy')
                if 'Wrapper' in label:
                    label += ')'
                labels.append(label)
                colors_list.append(COLORS.get(impl, '#999999'))

        bars = ax.barh(labels, overheads, color=colors_list)
        ax.set_xlabel('Overhead (%)', fontsize=12)
        ax.set_title('Secondary Query Overhead (10 queries)', fontsize=14)
        ax.grid(axis='x', alpha=0.3)
        ax.axvline(x=0, color='black', linestyle='-', linewidth=0.5)

        for bar, overhead in zip(bars, overheads):
            width = bar.get_width()
            ax.text(width, bar.get_y() + bar.get_height()/2,
                    f'{overhead:+.1f}%',
                    ha='left' if width >= 0 else 'right',
                    va='center', fontsize=10, fontweight='bold')

    plt.tight_layout()
    plt.savefig(OUTPUT_DIR / 'overhead_percentages.png', dpi=300, bbox_inches='tight')
    plt.close()

def generate_speedup_comparison(data: Dict):
    """Generate chart comparing loop vs bulk API speedups."""
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))
    fig.suptitle('Bulk API Speedup Factor', fontsize=16, fontweight='bold')

    # Insert speedup
    if 'wrapper_redb_loop' in data['insert'] and 'wrapper_redb_bulk' in data['insert']:
        ax = ax1
        sizes = [100, 1000]
        speedups = []

        for size in sizes:
            if size in data['insert']['wrapper_redb_loop'] and size in data['insert']['wrapper_redb_bulk']:
                loop_time = data['insert']['wrapper_redb_loop'][size]
                bulk_time = data['insert']['wrapper_redb_bulk'][size]
                speedup = loop_time / bulk_time
                speedups.append(speedup)
            else:
                speedups.append(0)

        bars = ax.bar([str(s) for s in sizes], speedups, color='#9467bd')
        ax.set_xlabel('Dataset Size', fontsize=12)
        ax.set_ylabel('Speedup Factor (×)', fontsize=12)
        ax.set_title('put_many() vs loop', fontsize=14)
        ax.grid(axis='y', alpha=0.3)
        ax.axhline(y=1, color='red', linestyle='--', linewidth=1, label='No speedup')
        ax.legend()

        for bar, speedup in zip(bars, speedups):
            height = bar.get_height()
            ax.text(bar.get_x() + bar.get_width()/2., height,
                    f'{speedup:.1f}×',
                    ha='center', va='bottom', fontsize=12, fontweight='bold')

    # Get speedup
    if 'wrapper_redb_loop' in data['get'] and 'wrapper_redb_bulk' in data['get']:
        ax = ax2
        loop_time = data['get']['wrapper_redb_loop']
        bulk_time = data['get']['wrapper_redb_bulk']
        speedup = loop_time / bulk_time

        bars = ax.bar(['get_many()'], [speedup], color='#9467bd')
        ax.set_ylabel('Speedup Factor (×)', fontsize=12)
        ax.set_title('get_many() vs loop', fontsize=14)
        ax.grid(axis='y', alpha=0.3)
        ax.axhline(y=1, color='red', linestyle='--', linewidth=1, label='No speedup')
        ax.legend()

        for bar in bars:
            height = bar.get_height()
            ax.text(bar.get_x() + bar.get_width()/2., height,
                    f'{speedup:.1f}×',
                    ha='center', va='bottom', fontsize=14, fontweight='bold')

    plt.tight_layout()
    plt.savefig(OUTPUT_DIR / 'bulk_api_speedup.png', dpi=300, bbox_inches='tight')
    plt.close()

def generate_absolute_performance_table(data: Dict):
    """Generate a markdown table with absolute performance numbers."""
    lines = []
    lines.append("# Benchmark Results Summary\n")
    lines.append("All times are mean values from Criterion benchmarks.\n")

    # Insert benchmarks
    lines.append("\n## Insert Performance\n")
    lines.append("| Implementation | 100 items | 1000 items |")
    lines.append("|----------------|-----------|------------|")

    for impl in sorted(data['insert'].keys()):
        time_100 = data['insert'][impl].get(100, 0) / 1_000_000  # Convert to ms
        time_1000 = data['insert'][impl].get(1000, 0) / 1_000_000
        lines.append(f"| {impl} | {time_100:.3f} ms | {time_1000:.3f} ms |")

    # Get benchmarks
    lines.append("\n## Get Performance (1000 items)\n")
    lines.append("| Implementation | Time |")
    lines.append("|----------------|------|")

    for impl in sorted(data['get'].keys()):
        time_us = data['get'][impl] / 1_000  # Convert to microseconds
        lines.append(f"| {impl} | {time_us:.2f} µs |")

    # Secondary query benchmarks
    lines.append("\n## Secondary Key Query Performance (10 queries)\n")
    lines.append("| Implementation | Time |")
    lines.append("|----------------|------|")

    for impl in sorted(data['secondary_query'].keys()):
        time_us = data['secondary_query'][impl] / 1_000  # Convert to microseconds
        lines.append(f"| {impl} | {time_us:.2f} µs |")

    # Bulk operations
    lines.append("\n## Bulk Operations (1000 items)\n")
    lines.append("| Implementation | Time |")
    lines.append("|----------------|------|")

    for impl in sorted(data['bulk'].keys()):
        time_ms = data['bulk'][impl] / 1_000_000  # Convert to ms
        lines.append(f"| {impl} | {time_ms:.3f} ms |")

    with open(OUTPUT_DIR / 'benchmark_summary.md', 'w') as f:
        f.write('\n'.join(lines))

def generate_raw_vs_zerocopy_comparison(data: Dict):
    """Generate charts comparing raw redb vs zerocopy redb."""
    if not data or len(data) == 0:
        print("No raw vs zerocopy data found, skipping...")
        return

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 7))
    fig.suptitle('Raw Redb vs ZeroCopy Redb Performance', fontsize=16, fontweight='bold')

    # Collect insert data
    insert_impls = {}
    for impl_name, impl_data in data.items():
        if 'insert' in impl_name and isinstance(impl_data, dict):
            insert_impls[impl_name] = impl_data

    if insert_impls:
        ax = ax1
        # Get all sizes
        all_sizes = set()
        for impl_data in insert_impls.values():
            all_sizes.update(impl_data.keys())
        sizes = sorted(list(all_sizes))

        # Plot lines for each implementation
        for impl_name, impl_data in sorted(insert_impls.items()):
            times_ms = []
            plot_sizes = []
            for size in sizes:
                if size in impl_data:
                    times_ms.append(impl_data[size] / 1_000_000)
                    plot_sizes.append(size)

            label = impl_name.replace('_', ' ').title()
            ax.plot(plot_sizes, times_ms, marker='o', linewidth=2, label=label, markersize=8)

        ax.set_xlabel('Dataset Size', fontsize=12)
        ax.set_ylabel('Time (ms)', fontsize=12)
        ax.set_title('Insert Performance', fontsize=14)
        ax.set_xscale('log')
        ax.set_yscale('log')
        ax.grid(True, alpha=0.3)
        ax.legend()

    # Collect read data
    read_impls = {}
    for impl_name, impl_data in data.items():
        if 'read' in impl_name and isinstance(impl_data, dict):
            read_impls[impl_name] = impl_data

    if read_impls:
        ax = ax2
        # Get all sizes
        all_sizes = set()
        for impl_data in read_impls.values():
            all_sizes.update(impl_data.keys())
        sizes = sorted(list(all_sizes))

        # Plot lines for each implementation
        for impl_name, impl_data in sorted(read_impls.items()):
            times_us = []
            plot_sizes = []
            for size in sizes:
                if size in impl_data:
                    times_us.append(impl_data[size] / 1_000)  # Convert to microseconds
                    plot_sizes.append(size)

            label = impl_name.replace('_', ' ').title().replace('Txn', 'Transaction')
            ax.plot(plot_sizes, times_us, marker='o', linewidth=2, label=label, markersize=8)

        ax.set_xlabel('Dataset Size', fontsize=12)
        ax.set_ylabel('Time (µs)', fontsize=12)
        ax.set_title('Read Performance', fontsize=14)
        ax.set_xscale('log')
        ax.set_yscale('log')
        ax.grid(True, alpha=0.3)
        ax.legend()

    plt.tight_layout()
    plt.savefig(OUTPUT_DIR / 'raw_vs_zerocopy_comparison.png', dpi=300, bbox_inches='tight')
    plt.close()

    # Generate overhead table
    lines = []
    lines.append("\n## Raw Redb vs ZeroCopy Overhead\n")

    if insert_impls and 'raw_insert' in insert_impls:
        lines.append("### Insert Operations\n")
        lines.append("| Size | Raw Redb | ZeroCopy (loop) | ZeroCopy (bulk) | Loop Overhead | Bulk Overhead |")
        lines.append("|------|----------|-----------------|-----------------|---------------|---------------|")

        raw_data = insert_impls.get('raw_insert', {})
        zc_data = insert_impls.get('zerocopy_insert', {})
        zc_bulk_data = insert_impls.get('zerocopy_bulk_insert', {})

        for size in sorted(raw_data.keys()):
            raw_time = raw_data[size] / 1_000_000
            zc_time = zc_data.get(size, 0) / 1_000_000
            zc_bulk_time = zc_bulk_data.get(size, 0) / 1_000_000

            loop_overhead = ((zc_time - raw_time) / raw_time * 100) if raw_time > 0 else 0
            bulk_overhead = ((zc_bulk_time - raw_time) / raw_time * 100) if raw_time > 0 else 0

            lines.append(f"| {size} | {raw_time:.3f} ms | {zc_time:.3f} ms | {zc_bulk_time:.3f} ms | {loop_overhead:+.1f}% | {bulk_overhead:+.1f}% |")

    return '\n'.join(lines)

def main():
    print("Parsing benchmark data...")
    data = parse_cross_store_benchmarks()

    print("Generating insert comparison bar chart...")
    generate_insert_comparison_bar_chart(data['insert'])

    print("Generating overhead percentage charts...")
    generate_overhead_percentage_chart(data)

    print("Generating speedup comparison charts...")
    generate_speedup_comparison(data)

    print("Generating raw vs zerocopy comparison...")
    raw_vs_zc_table = generate_raw_vs_zerocopy_comparison(data['raw_vs_zerocopy'])

    print("Generating performance summary table...")
    generate_absolute_performance_table(data)

    # Append raw vs zerocopy data to summary
    if raw_vs_zc_table:
        with open(OUTPUT_DIR / 'benchmark_summary.md', 'a') as f:
            f.write('\n' + raw_vs_zc_table)

    print(f"\nCharts generated in {OUTPUT_DIR}/")
    print("- insert_comparison_bars.png")
    print("- overhead_percentages.png")
    print("- bulk_api_speedup.png")
    print("- raw_vs_zerocopy_comparison.png")
    print("- benchmark_summary.md")

if __name__ == "__main__":
    main()
