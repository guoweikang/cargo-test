#!/bin/bash
# Test script for cargo-kbuild validation mechanism

set -e

echo "=============================================="
echo "🧪 Cargo-Kbuild Validation Tests"
echo "=============================================="
echo ""

# Cleanup
rm -rf .cargo/config.toml target/kbuild

# Build cargo-kbuild first
echo "Step 0: Building cargo-kbuild tool"
echo "----------------------------------------------------"
cargo build -p cargo-kbuild
echo ""

echo "Test 1: ✅ Auto-generate .cargo/config.toml"
echo "----------------------------------------------------"
./target/debug/cargo-kbuild build --kconfig .config

if [ -f ".cargo/config.toml" ]; then
    echo "✅ .cargo/config.toml generated successfully"
    echo ""
    echo "Generated content:"
    cat .cargo/config.toml
else
    echo "❌ .cargo/config.toml not found"
    exit 1
fi
echo ""

echo "Test 2: ✅ Verify CONFIG_* declarations"
echo "----------------------------------------------------"
if grep -q "CONFIG_SMP" .cargo/config.toml && \
   grep -q "CONFIG_NET" .cargo/config.toml && \
   grep -q "CONFIG_ASYNC" .cargo/config.toml; then
    echo "✅ All expected CONFIG_* declarations found"
else
    echo "❌ Missing CONFIG_* declarations"
    exit 1
fi
echo ""

echo "Test 3: ✅ Generate config.rs with constants"
echo "----------------------------------------------------"
if [ -f "target/kbuild/config.rs" ]; then
    echo "✅ config.rs generated"
    echo ""
    echo "Generated constants:"
    cat target/kbuild/config.rs
else
    echo "❌ config.rs not found"
    exit 1
fi
echo ""

echo "Test 4: ✅ Build succeeds with zero warnings"
echo "----------------------------------------------------"
cargo build 2>&1 | tee /tmp/build.log

if grep -q "warning.*unexpected.*cfg" /tmp/build.log; then
    echo "❌ Found unexpected cfg warnings"
    exit 1
else
    echo "✅ No cfg warnings - build clean!"
fi
echo ""

# Save original Cargo.toml
cp crates/kernel_net/Cargo.toml crates/kernel_net/Cargo.toml.backup

echo "Test 5: ✅ Correct configuration (should pass)"
echo "----------------------------------------------"
./target/debug/cargo-kbuild build --kconfig .config 2>&1 | grep -E "(✅|❌)" || true
echo ""

echo "Test 6: ❌ Incorrect configuration (should fail)"
echo "-----------------------------------------------"
# Modify Cargo.toml to add wrong sub-feature
sed -i 's/CONFIG_NET = \[\]/CONFIG_NET = ["network_utils\/CONFIG_ASYNC"]/' crates/kernel_net/Cargo.toml
echo "Modified kernel_net/Cargo.toml to use network_utils/CONFIG_ASYNC"
echo ""

if ./target/debug/cargo-kbuild build --kconfig .config 2>&1 | grep -q "Error in crate 'kernel_net'"; then
    echo "✅ Test passed: Validation correctly rejected sub-feature for kbuild-enabled dependency"
else
    echo "❌ Test failed: Validation should have rejected the configuration"
fi
echo ""

# Restore original Cargo.toml
mv crates/kernel_net/Cargo.toml.backup crates/kernel_net/Cargo.toml

echo "Test 7: ✅ Restored configuration (should pass again)"
echo "----------------------------------------------------"
./target/debug/cargo-kbuild build --kconfig .config 2>&1 | grep -E "(✅|❌)" || true
echo ""

echo "=============================================="
echo "🎉 All tests completed"
echo "=============================================="
