#!/bin/bash
# Test script for cargo-kbuild validation mechanism

set -e

echo "🧪 Testing Cargo-Kbuild Validation Mechanism"
echo "=============================================="
echo

# Save original Cargo.toml
cp crates/kernel_net/Cargo.toml crates/kernel_net/Cargo.toml.backup

echo "Test 1: ✅ Correct configuration (should pass)"
echo "----------------------------------------------"
./target/debug/cargo-kbuild build --kconfig .config 2>&1 | grep -E "(✅|❌)" || true
echo

echo "Test 2: ❌ Incorrect configuration (should fail)"
echo "-----------------------------------------------"
# Modify Cargo.toml to add wrong sub-feature
sed -i 's/CONFIG_NET = \[\]/CONFIG_NET = ["network_utils\/CONFIG_ASYNC"]/' crates/kernel_net/Cargo.toml
echo "Modified kernel_net/Cargo.toml to use network_utils/CONFIG_ASYNC"
echo

if ./target/debug/cargo-kbuild build --kconfig .config 2>&1 | grep -q "Error in crate 'kernel_net'"; then
    echo "✅ Test passed: Validation correctly rejected sub-feature for kbuild-enabled dependency"
else
    echo "❌ Test failed: Validation should have rejected the configuration"
fi
echo

# Restore original Cargo.toml
mv crates/kernel_net/Cargo.toml.backup crates/kernel_net/Cargo.toml

echo "Test 3: ✅ Restored configuration (should pass again)"
echo "----------------------------------------------------"
./target/debug/cargo-kbuild build --kconfig .config 2>&1 | grep -E "(✅|❌)" || true
echo

echo "=============================================="
echo "🎉 All tests completed"
