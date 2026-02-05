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

echo "Test 4: 🔢 Config.rs generation"
echo "----------------------------------------------------"
# Verify target/kbuild/config.rs is generated
if [ -f "target/kbuild/config.rs" ]; then
    echo "✅ config.rs generated successfully"
    echo "Contents:"
    cat target/kbuild/config.rs
    echo
    
    # Verify expected constants
    if grep -q "CONFIG_LOG_LEVEL: i32 = 3" target/kbuild/config.rs; then
        echo "✅ CONFIG_LOG_LEVEL constant found"
    else
        echo "❌ CONFIG_LOG_LEVEL constant missing"
    fi
    
    if grep -q "CONFIG_MAX_CPUS: i32 = 8" target/kbuild/config.rs; then
        echo "✅ CONFIG_MAX_CPUS constant found"
    else
        echo "❌ CONFIG_MAX_CPUS constant missing"
    fi
    
    if grep -q 'CONFIG_DEFAULT_SCHEDULER: &str = "cfs"' target/kbuild/config.rs; then
        echo "✅ CONFIG_DEFAULT_SCHEDULER constant found"
    else
        echo "❌ CONFIG_DEFAULT_SCHEDULER constant missing"
    fi
else
    echo "❌ config.rs not found"
fi
echo

echo "Test 5: 📦 Demo mixed deps crate"
echo "----------------------------------------------------"
if cargo build -p demo_mixed_deps 2>&1 | grep -q "Finished"; then
    echo "✅ demo_mixed_deps crate builds successfully"
else
    echo "❌ demo_mixed_deps crate build failed"
fi
echo

echo "=============================================="
echo "🎉 All tests completed"
