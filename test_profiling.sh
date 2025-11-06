#!/bin/bash
echo "Testing flamegraph generation..."
cd /home/rusta/Projects/NewsNet/netabase_store
cargo bench --bench sled_wrapper_overhead -- --profile-time=3 insert/wrapper/100 2>&1 | grep -E "(Profiling|Complete)" | head -n 5
echo ""
echo "Checking for flamegraph..."
if [ -f target/criterion/insert/wrapper/100/profile/flamegraph.svg ]; then
    size=$(du -h target/criterion/insert/wrapper/100/profile/flamegraph.svg | cut -f1)
    echo "✓ Flamegraph generated: $size"
    echo "  Path: target/criterion/insert/wrapper/100/profile/flamegraph.svg"
else
    echo "✗ No flamegraph found"
fi
