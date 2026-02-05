# Cargo-Kbuild MVP: Intelligent Sub-feature Validation System

🚀 **A proof-of-concept implementation of intelligent Kconfig-style build system for Rust projects**

## 📖 Overview

Cargo-Kbuild is an intelligent build tool that brings Linux kernel-style Kconfig to Rust projects. It intelligently validates feature dependencies and dynamically adjusts constraint rules based on whether dependency packages support kbuild.

### Key Features

- ✅ **Intelligent Validation**: Automatically detects kbuild-enabled dependencies
- ✅ **Smart Constraint Rules**: Allows sub-features for third-party libraries, blocks them for kbuild-enabled dependencies
- ✅ **Clear Error Messages**: Provides actionable feedback with solutions
- ✅ **Seamless Integration**: Works with existing Cargo workflows
- ✅ **Parallel Access Architecture**: All crates read from a global `.config` file independently

## 🎯 Core Concept: Intelligent Sub-feature Validation

### The Problem

In a kernel-style build system, all modules should read their configuration from a global `.config` file. However, not all dependencies support this pattern:

- **Internal kbuild-enabled crates**: Should read `.config` themselves
- **Third-party libraries**: Cannot be modified, must be controlled via features
- **Legacy code**: May not have kbuild support yet

### The Solution

Cargo-kbuild **intelligently distinguishes** between dependency types and applies appropriate rules:

```rust
if dependency_supports_kbuild {
    // ❌ Reject: Don't specify sub-features for kbuild-enabled dependencies
    // They should read .config themselves
    return Error;
} else {
    // ✅ Allow: Third-party libraries need feature specification
    return Ok;
}
```

## 🔍 Validation Rules

| Dependency Type | Supports Kbuild? | Detection Method | Sub-feature Allowed? | Example |
|----------------|------------------|------------------|---------------------|---------|
| **Kbuild-enabled internal lib** | ✅ Yes | `metadata.kbuild.enabled = true`<br>OR has `CONFIG_*` features | ❌ No | `CONFIG_NET = ["network_utils"]` ✅<br>`CONFIG_NET = ["network_utils/async"]` ❌ |
| **Third-party library** | ❌ No | No metadata.kbuild<br>No CONFIG_* features | ✅ Yes | `CONFIG_LOGGING = ["log/std"]` ✅ |
| **Legacy code (not migrated)** | ❌ No | No metadata.kbuild | ✅ Yes | `CONFIG_LEGACY = ["legacy_module/usb"]` ✅ |

## 🏗️ Architecture

### Parallel Access Pattern

```
.config (Global Configuration Source)
   ↓ Parallel Access
   ├─→ kernel_net     → Reads CONFIG_NET
   ├─→ network_utils  → Reads CONFIG_ASYNC
   ├─→ kernel_task    → Reads CONFIG_SMP
   └─→ kernel_irq     → Reads CONFIG_SMP
   
Third-party libraries (e.g., log):
   ↑ Controlled by parent
   └── kernel_net specifies "log/std"
```

## 🚀 Quick Start

### 1. Installation

```bash
# Clone the repository
git clone https://github.com/guoweikang/cargo-test.git
cd cargo-test

# Build cargo-kbuild tool
cargo build -p cargo-kbuild
```

### 2. Create Configuration

Create a `.config` file in your project root:

```bash
# Kernel Configuration
CONFIG_SMP=y
CONFIG_PREEMPT=y
CONFIG_NET=y
CONFIG_ASYNC=y
CONFIG_LOGGING=y
CONFIG_DEBUG=n
```

### 3. Mark Crates as Kbuild-Enabled

In your crate's `Cargo.toml`:

```toml
[package]
name = "my_crate"
version = "0.1.0"

[package.metadata.kbuild]
enabled = true  # ← Mark as kbuild-enabled

[features]
CONFIG_SMP = []
CONFIG_NET = []
```

### 4. Build with Cargo-Kbuild

```bash
# Validate and build
./target/debug/cargo-kbuild build --kconfig .config
```

## 📋 Example: Mixed Dependencies

### Scenario

A network subsystem that:
- Depends on a kbuild-enabled internal library (`network_utils`)
- Depends on a third-party library (`log`)

### Implementation

```toml
# crates/kernel_net/Cargo.toml
[package]
name = "kernel_net"

[package.metadata.kbuild]
enabled = true

[dependencies]
network_utils = { path = "../network_utils" }
log = { version = "0.4", optional = true }

[features]
# ✅ Correct: network_utils supports kbuild, only declare dependency
CONFIG_NET = []

# ✅ Correct: log doesn't support kbuild, can specify sub-feature
CONFIG_LOGGING = ["log/std"]
```

```toml
# crates/network_utils/Cargo.toml
[package]
name = "network_utils"

[package.metadata.kbuild]
enabled = true  # ← Supports kbuild

[features]
CONFIG_ASYNC = []
```

### What Happens?

#### ✅ Correct Configuration

```toml
CONFIG_NET = []  # No sub-feature for kbuild-enabled dependency
```

**Output:**
```
✅ Feature validation passed!
```

#### ❌ Incorrect Configuration

```toml
CONFIG_NET = ["network_utils/async"]  # ❌ Trying to control kbuild-enabled dep
```

**Output:**
```
❌ Error in crate 'kernel_net':

Feature 'CONFIG_NET' specifies sub-feature: 'network_utils/async'

Dependency 'network_utils' has kbuild enabled:
  - It should control its own features by reading .config
  - Cannot be controlled by dependent crates

Expected: 'network_utils'
Found:    'network_utils/async'

Solution:
1. Change 'network_utils/async' to 'network_utils' in [features]
2. Ensure 'network_utils' reads CONFIG_ASYNC from .config

Or, if 'network_utils' should NOT use kbuild:
  - Remove [package.metadata.kbuild] from network_utils/Cargo.toml
  - Remove CONFIG_* features from network_utils
```

## 🔢 Complex Configuration Types

Beyond boolean flags (y/n), cargo-kbuild supports integer and string configuration values.

### Integer and String Configs

**In .config:**
```bash
CONFIG_SMP=y
CONFIG_LOG_LEVEL=3
CONFIG_MAX_CPUS=8
CONFIG_DEFAULT_SCHEDULER="cfs"
```

**Generated config.rs:**
```rust
// Auto-generated by cargo-kbuild from .config
// DO NOT EDIT MANUALLY

#[allow(dead_code)]
pub const CONFIG_LOG_LEVEL: i32 = 3;

#[allow(dead_code)]
pub const CONFIG_MAX_CPUS: i32 = 8;

#[allow(dead_code)]
pub const CONFIG_DEFAULT_SCHEDULER: &str = "cfs";
```

**Usage Example:**
```rust
use kbuild_config::*;

pub fn setup_logging() {
    if CONFIG_LOG_LEVEL >= 2 {
        println!("Debug logging enabled");
    }
    
    if CONFIG_MAX_CPUS > 4 {
        println!("Multi-core system detected: {} CPUs", CONFIG_MAX_CPUS);
    }
    
    println!("Using scheduler: {}", CONFIG_DEFAULT_SCHEDULER);
}
```

### Type Detection Rules

- **Boolean**: Values `y`, `n`, or `m` → Handled via `--cfg` flags (not in config.rs)
- **Integer**: Numeric values without quotes → `i32` constants
- **String**: Values in double quotes → `&str` constants

## 📦 Third-Party Dependencies with Sub-features

Cargo-kbuild intelligently distinguishes between different types of dependencies:

### ✅ Allowed: Third-party crates can specify sub-features

```toml
[dependencies]
log = { version = "0.4", optional = true }
tokio = { version = "1.0", optional = true }

[features]
# ✅ Third-party crates - sub-features are allowed
CONFIG_LOGGING = ["log/std"]
CONFIG_ASYNC = ["tokio/rt-multi-thread"]
```

### ❌ Not Allowed: kbuild-enabled workspace crates cannot specify sub-features

```toml
[dependencies]
network_utils = { path = "../network_utils" }  # kbuild-enabled

[features]
# ❌ Error! kbuild-enabled crates should read .config directly
CONFIG_NET = ["network_utils/CONFIG_ASYNC"]
```

**Why?** Kbuild-enabled crates read their configuration from `.config` directly. Parent crates cannot override this behavior through sub-features.

**Solution:**
1. Change to: `CONFIG_NET = ["network_utils"]`
2. Enable `CONFIG_ASYNC` in the `.config` file

### Validation Behavior

When cargo-kbuild encounters a sub-feature specification:

1. **If dependency is kbuild-enabled workspace crate** → ❌ Error with clear message
2. **If dependency is non-kbuild workspace crate** → ℹ️ Info message, allowed
3. **If dependency is third-party crate** → ℹ️ Info message, allowed


## 🎪 Running the Demo

The repository includes a complete demo application:

```bash
# Build with cargo-kbuild
./target/debug/cargo-kbuild build --kconfig .config

# Run the demo
./target/debug/cargo-test
```

**Expected Output:**

> **Note**: The demo application uses Chinese text in output messages to demonstrate internationalization support. The functionality remains the same regardless of language.

```
🚀 ============================================
🚀  Cargo-Kbuild MVP Demo
🚀 ============================================

🔄 [SCHEDULE] 调度器初始化
⚡ [IRQ] 中断子系统初始化
⚡ [IRQ] SMP 中断路由已启用
📋 [TASK] SMP 任务系统初始化
🔄 [SCHEDULE] SMP 调度器已启用
🔄 [SCHEDULE] 抢占式调度已启用

📋 [TASK] 创建任务 1 (绑定到 CPU 0)
📋 [TASK] 创建任务 2 (绑定到 CPU 1)
🔄 [SCHEDULE] 调度任务 1 到 CPU 0
🔄 [SCHEDULE] 调度任务 2 到 CPU 1

🌐 [NET] 网络子系统初始化
🔧 [NETWORK_UTILS] 初始化网络工具
🔧 [NETWORK_UTILS] 异步网络支持已启用
🌐 [NET] 网络工具库已加载
📝 [NET] 日志系统已启用

🚗 [LEGACY] 传统驱动初始化

🎉 ============================================
🎉  系统初始化完成
🎉 ============================================
```

## 📂 Project Structure

```
cargo-test/
├── .config                          # Global kernel configuration
├── Cargo.toml                       # Workspace root
├── src/
│   └── main.rs                      # Demo application
├── cargo-kbuild/                    # Build tool
│   └── src/
│       └── main.rs                  # Intelligent validation logic
├── crates/
│   ├── kernel_irq/                  # Interrupt handling (kbuild)
│   ├── kernel_task/                 # Task management (kbuild)
│   ├── kernel_schedule/             # Scheduler (kbuild)
│   ├── kernel_net/                  # Network subsystem (kbuild + mixed deps)
│   ├── network_utils/               # Network utilities (kbuild)
│   └── legacy_driver/               # Legacy driver (non-kbuild)
└── README.md
```

## 🔧 How It Works

### 1. Workspace Parsing

Cargo-kbuild parses all crates in the workspace and builds a dependency graph:

```rust
struct CrateInfo {
    name: String,
    has_kbuild: bool,
    features: HashMap<String, Vec<String>>,
}
```

### 2. Kbuild Detection

For each dependency, it checks:

```rust
fn is_dependency_kbuild_enabled(workspace: &Workspace, pkg_name: &str) -> bool {
    if let Some(dep_crate) = workspace.find_crate(pkg_name) {
        // Method 1: Check metadata
        if dep_crate.has_kbuild {
            return true;
        }
        
        // Method 2: Check for CONFIG_* features
        if dep_crate.features.keys().any(|f| f.starts_with("CONFIG_")) {
            return true;
        }
    }
    false
}
```

### 3. Feature Validation

For each CONFIG_* feature, validate dependencies:

```rust
for dep in feature_dependencies {
    if let Some((pkg_name, sub_feature)) = dep.split_once('/') {
        if is_dependency_kbuild_enabled(workspace, pkg_name) {
            return Err("Cannot specify sub-feature for kbuild-enabled dependency");
        } else {
            println!("ℹ️  Third-party library, sub-feature allowed");
        }
    }
}
```

### 4. Configuration Application

Read `.config` and apply as compiler flags:

```rust
// Read .config
CONFIG_SMP=y → Enable CONFIG_SMP
CONFIG_NET=y → Enable CONFIG_NET

// Apply as RUSTFLAGS
RUSTFLAGS="--cfg CONFIG_SMP --cfg CONFIG_NET" cargo build
```

## 💡 Design Principles

### 1. **Intelligent, Not Restrictive**

Traditional approach: "Never allow sub-features"  
Our approach: "Allow when appropriate, block when necessary"

### 2. **Clear Error Messages**

Every error includes:
- What went wrong
- Why it's wrong
- How to fix it
- Alternative solutions

### 3. **Gradual Migration**

- New crates can adopt kbuild
- Old crates continue to work
- Third-party libraries just work

### 4. **Zero Runtime Overhead**

All validation happens at build time, with no runtime cost.

## 🎯 Technical Advantages

| Advantage | Description |
|-----------|-------------|
| **Intelligence** | Automatically detects dependency types |
| **Flexibility** | Supports old and new crates coexisting |
| **Compatibility** | Works with unmodifiable third-party libraries |
| **Clarity** | Clear error messages with solutions |
| **Scalability** | Efficient validation for large projects |

## 🛠️ Advanced Usage

### Custom Config Location

```bash
./target/debug/cargo-kbuild build --kconfig path/to/my.config
```

### Integration with CI/CD

```yaml
# .github/workflows/build.yml
- name: Build with cargo-kbuild
  run: |
    cargo build -p cargo-kbuild
    ./target/debug/cargo-kbuild build --kconfig .config
```

### Testing Different Configurations

```bash
# Test with minimal config
echo "CONFIG_SMP=n" > .config.minimal
./target/debug/cargo-kbuild build --kconfig .config.minimal

# Test with full features
echo "CONFIG_SMP=y\nCONFIG_NET=y" > .config.full
./target/debug/cargo-kbuild build --kconfig .config.full
```

## 🧪 Testing

### Manual Testing

```bash
# 1. Test successful validation
./target/debug/cargo-kbuild build --kconfig .config

# 2. Test error detection (modify a Cargo.toml to add wrong sub-feature)
# Edit crates/kernel_net/Cargo.toml:
#   CONFIG_NET = ["network_utils/async"]  # ❌ Should fail
./target/debug/cargo-kbuild build --kconfig .config
```

### Expected Behavior

| Test Case | Expected Result |
|-----------|----------------|
| Kbuild-enabled dep without sub-feature | ✅ Pass |
| Kbuild-enabled dep with sub-feature | ❌ Clear error message |
| Third-party dep with sub-feature | ✅ Pass with info message |
| Non-existent config file | ❌ Error: Config not found |

## 🔮 Future Enhancements

### Planned Features

- [ ] **Interactive Config Editor**: TUI for editing `.config`
- [ ] **Dependency Visualization**: Show feature dependency graph
- [ ] **Auto-migration Tool**: Convert existing projects to kbuild
- [ ] **IDE Integration**: VSCode plugin for validation
- [ ] **Config Templates**: Pre-defined configuration profiles

### Potential Improvements

- [ ] Support for conditional dependencies
- [ ] Nested workspace validation
- [ ] Performance optimization for large workspaces
- [ ] Config file includes and composition

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. **Report Issues**: Found a bug? Open an issue
2. **Suggest Features**: Have an idea? Let's discuss
3. **Submit PRs**: Code contributions are appreciated
4. **Improve Documentation**: Help make this clearer

## 📄 License

This project is part of the cargo-test repository.

## 🙏 Acknowledgments

- Inspired by Linux kernel's Kconfig system
- Built with Rust and the Cargo ecosystem
- Thanks to all contributors and testers

## 📚 Learn More

### Key Concepts

- **Kconfig**: Linux kernel configuration system
- **Cargo Features**: Rust's conditional compilation
- **Workspace**: Multi-crate Rust projects
- **Metadata**: Custom package metadata in Cargo.toml

### Related Resources

- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Linux Kconfig](https://www.kernel.org/doc/html/latest/kbuild/kconfig.html)
- [Rust Conditional Compilation](https://doc.rust-lang.org/reference/conditional-compilation.html)

---

**Made with ❤️ for better Rust project configuration management**
