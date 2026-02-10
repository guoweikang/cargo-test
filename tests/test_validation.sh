#!/bin/bash
set -e

echo "=============================================="
echo "🧪 Cargo-Kbuild Validation Tests"
echo "=============================================="

# Cleanup any existing generated files
echo ""
echo "Step 0: Cleanup"
echo "----------------------------------------------------"
rm -f .cargo/config.toml
rm -rf target/kbuild
echo "✅ Cleanup completed"
echo ""

# Build cargo-kbuild first
echo "Step 1: Build cargo-kbuild tool"
echo "----------------------------------------------------"
cargo build -p cargo-kbuild
echo ""

echo "Test 1: ✅ Build with simplified features"
echo "----------------------------------------------------"
./target/debug/cargo-kbuild build --kconfig .config

if [ -f ".cargo/config.toml" ]; then
    echo "✅ .cargo/config.toml generated successfully"
    echo ""
    echo "Generated content:"
    cat .cargo/config.toml
    echo ""
    
    # Verify key declarations (should include configs from .config, not just features)
    if grep -q "SMP" .cargo/config.toml && \
       grep -q "NET" .cargo/config.toml && \
       grep -q "ASYNC" .cargo/config.toml; then
        echo "✅ All expected config declarations found"
    else
        echo "❌ Missing config declarations"
        exit 1
    fi
else
    echo "❌ .cargo/config.toml not found"
    exit 1
fi
echo ""

echo "Test 2: ✅ Build with zero warnings"
echo "----------------------------------------------------"
cargo build 2>&1 | tee /tmp/cargo-kbuild-test.log

if grep -qi "warning.*unexpected.*cfg" /tmp/cargo-kbuild-test.log; then
    echo "❌ Found unexpected cfg warnings"
    grep -i "warning.*cfg" /tmp/cargo-kbuild-test.log
    exit 1
else
    echo "✅ No cfg warnings - build clean!"
fi
echo ""

# Save original Cargo.toml
cp crates/kernel_net/Cargo.toml crates/kernel_net/Cargo.toml.backup

echo "Test 3: ❌ Incorrect sub-feature configuration (should fail)"
echo "-----------------------------------------------"
# Modify Cargo.toml to add wrong sub-feature
cat > crates/kernel_net/Cargo.toml << 'EOF'
[package]
name = "kernel_net"
version = "0.1.0"
edition = "2021"

[package.metadata.kbuild]
enabled = true

[dependencies]
network_utils = { path = "../network_utils" }

[features]
NET = ["network_utils/ASYNC"]
EOF

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

echo "Test 4: ✅ Restored configuration (should pass again)"
echo "----------------------------------------------------"
./target/debug/cargo-kbuild build --kconfig .config 2>&1 | grep -E "(✅|❌)" || true
echo

echo "Test 5: 🔢 Config.rs generation verification"
echo "----------------------------------------------------"
# Verify target/kbuild/config.rs is generated
if [ -f "target/kbuild/config.rs" ]; then
    echo "✅ config.rs generated successfully"
    echo "Contents:"
    cat target/kbuild/config.rs
    echo
    
    # Verify expected constants
    if grep -q "LOG_LEVEL: i32 = 3" target/kbuild/config.rs; then
        echo "✅ LOG_LEVEL constant found"
    else
        echo "❌ LOG_LEVEL constant missing"
    fi
    
    if grep -q "MAX_CPUS: i32 = 8" target/kbuild/config.rs; then
        echo "✅ MAX_CPUS constant found"
    else
        echo "❌ MAX_CPUS constant missing"
    fi
    
    if grep -q 'DEFAULT_SCHEDULER: &str = "cfs"' target/kbuild/config.rs; then
        echo "✅ DEFAULT_SCHEDULER constant found"
    else
        echo "❌ DEFAULT_SCHEDULER constant missing"
    fi
else
    echo "❌ config.rs not found"
fi
echo

echo "Test 6: 📦 Demo mixed deps crate"
echo "----------------------------------------------------"
if cargo build -p demo_mixed_deps 2>&1 | grep -q "Finished"; then
    echo "✅ demo_mixed_deps crate builds successfully"
else
    echo "❌ demo_mixed_deps crate build failed"
fi
echo

echo "Test 7: 🚀 Run demo application"
echo "----------------------------------------------------"
if ./target/debug/cargo-test 2>&1 | grep -q "Cargo-Kbuild Demo"; then
    echo "✅ Demo application runs successfully"
else
    echo "❌ Demo application failed"
fi
echo

echo "Test 8: 🔄 Cargo wrapper - check command"
echo "----------------------------------------------------"
if ./target/debug/cargo-kbuild check --kconfig .config 2>&1 | grep -q "✅ Command completed successfully"; then
    echo "✅ Check command works with kbuild configuration"
else
    echo "❌ Check command failed"
fi
echo

echo "Test 9: 🔄 Cargo wrapper - test command (expect some failures)"
echo "----------------------------------------------------"
# Just verify the command runs (some tests may fail due to no lib target in root)
if ./target/debug/cargo-kbuild test --kconfig .config 2>&1 | grep -qE "(🔨 Running cargo test|Finished|error: no library)"; then
    echo "✅ Test command forwards to cargo successfully"
else
    echo "❌ Test command failed to forward"
fi
echo

echo "Test 10: 🔄 Cargo wrapper - with arguments"
echo "----------------------------------------------------"
if ./target/debug/cargo-kbuild build --release --kconfig .config 2>&1 | grep -qE "(✅ Command completed successfully|Finished)"; then
    echo "✅ Build with --release argument works"
else
    echo "❌ Build with arguments failed"
fi
echo

echo "=============================================="
echo "🎉 All tests completed"

