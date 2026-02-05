# cargo-kbuild

在 Rust 项目中使用 Kconfig 风格的全局配置。

## 为什么需要这个工具？

Cargo 的 feature 是树状传递的：父包开启 feature，子包被动接收。

但内核开发不是这样的：所有模块平等地读取同一份配置文件。

cargo-kbuild 实现了这种"全局配置"模式。

## 架构概述

**正确的理解：**
- `.config` 文件由外部 Kconfig 工具生成（如 Linux 的 `make menuconfig`）
- cargo-kbuild 读取现有的 `.config`，生成 `config.rs`，设置 RUSTFLAGS，并调用 cargo
- Cargo.toml 中的 `CONFIG_*` features **仅用于可选依赖**，不用于声明配置使用
- 任何 kbuild 启用的 crate 都可以使用任何 `CONFIG_*`，无需在 Cargo.toml 中声明

**错误的理解 ❌：**
- ❌ cargo-kbuild 生成 `.config` 文件
- ❌ 需要在 Cargo.toml 中声明所有使用的 `CONFIG_*`
- ❌ Features 用于声明哪些配置被代码使用

## 快速开始

安装：
```bash
cargo install --path cargo-kbuild
```

创建 `.config` 文件（使用外部工具或手动创建）：
```
CONFIG_SMP=y
CONFIG_NET=y
CONFIG_LOG_LEVEL=3
```

编译：
```bash
cargo-kbuild build
```

## 工作原理

1. 读取 `.config` 文件（应由 Kconfig 工具生成）
2. 生成 `target/kbuild/config.rs` 包含配置常量
3. 生成 `.cargo/config.toml` 声明所有 CONFIG_* 供 check-cfg lint 使用
4. 设置 RUSTFLAGS 启用配置选项
5. 验证依赖关系（防止 kbuild crate 指定子特性）
6. 调用 cargo 进行编译

不需要写 build.rs，不需要手动管理 cfg 声明。

## 特性声明原则

### 何时需要声明 CONFIG_* features

**只有在有可选依赖时才需要：**

```toml
# ✅ 正确：有可选依赖，需要声明 feature
[dependencies]
kernel_net = { path = "crates/kernel_net", optional = true }

[features]
CONFIG_NET = ["kernel_net"]  # 启用可选依赖
```

**代码中使用 CONFIG_* 不需要声明：**

```toml
# ✅ 正确：代码使用 CONFIG_SMP，但无可选依赖，不需要声明
[package.metadata.kbuild]
enabled = true

# 不需要 [features] 部分
```

```rust
// 代码中可以直接使用，无需在 Cargo.toml 中声明
#[cfg(CONFIG_SMP)]
fn init_smp() {
    println!("SMP enabled");
}
```

### 示例对比

❌ **错误方式**（旧的做法）：
```toml
# 不要这样做：为代码中使用的配置声明 features
[features]
CONFIG_SMP = []
CONFIG_PREEMPT = []
CONFIG_LOGGING = []
```

✅ **正确方式**（新的架构）：
```toml
# 方式 1：没有可选依赖，完全不需要 features
[package.metadata.kbuild]
enabled = true

# 方式 2：有可选依赖，只声明管理依赖的 features
[dependencies]
tokio = { version = "1.0", optional = true }

[features]
CONFIG_ASYNC = ["tokio"]  # 仅因为需要启用 tokio 依赖
```

## 示例

参考 `crates/` 目录中的示例 crates：
- `kernel_irq/` - 无可选依赖，无 features 声明
- `kernel_task/` - 无可选依赖，无 features 声明
- `kernel_schedule/` - 无可选依赖，无 features 声明
- `kernel_net/` - 作为可选依赖被根 crate 使用
- `demo_mixed_deps/` - 混合使用第三方库和配置常量

## 命令

```bash
cargo-kbuild build              # 编译项目
cargo-kbuild build --kconfig custom.config  # 使用自定义配置文件
cargo-kbuild --help             # 显示帮助
cargo-kbuild --version          # 显示版本
```

**注意：** 不再有 `init` 和 `check` 命令。`.config` 应由外部 Kconfig 工具生成。

## 文档

- [用户指南](cargo-kbuild/README.md) - 详细使用说明
- [实现细节](IMPLEMENTATION_DETAILS.md) - 技术架构
- [实现总结](IMPLEMENTATION_SUMMARY.md) - 功能概览

## 核心特性

### 智能特性验证

cargo-kbuild 会自动检测 crate 是否支持 kbuild：

- **支持 kbuild 的内部 crate**：不能指定子特性，必须自己读取 `.config`
- **第三方库**：可以指定子特性（如 `log/std`）
- **未迁移的旧代码**：可以用传统方式控制

### 全局配置访问

所有启用 kbuild 的 crate 都可以访问 `.config` 中的任何配置：

```rust
// 任何 kbuild 启用的 crate 都可以使用任何 CONFIG_*
// 无需在 Cargo.toml 中声明
#[cfg(CONFIG_SMP)]
fn init_smp() {
    println!("SMP enabled");
}

#[cfg(CONFIG_PREEMPT)]
fn enable_preemption() {
    println!("Preemption enabled");
}
```

### 自动化

- 自动生成 `.cargo/config.toml` 声明所有 CONFIG_* 供 check-cfg lint 使用
- 自动验证特性依赖关系（防止 kbuild crate 指定子特性）
- 自动从 `.config` 生成 `config.rs` 常量文件
- 零编译警告

## 启用 kbuild

在 crate 的 `Cargo.toml` 中添加：

```toml
[package.metadata.kbuild]
enabled = true
```

**不需要声明 CONFIG_* features，除非有可选依赖。**

## 使用配置值

### 布尔配置

在 `.config` 中：
```
CONFIG_SMP=y
CONFIG_PREEMPT=n
```

在代码中：
```rust
#[cfg(CONFIG_SMP)]
fn init_smp() {
    println!("SMP enabled");
}

#[cfg(not(CONFIG_SMP))]
fn init_single_core() {
    println!("Single core mode");
}
```

### 数值和字符串配置

在 `.config` 中：
```
CONFIG_LOG_LEVEL=3
CONFIG_MAX_CPUS=8
CONFIG_DEFAULT_SCHEDULER="cfs"
```

在代码中：
```rust
// 需要 crate 依赖 kbuild_config
use kbuild_config::*;

fn init() {
    println!("Log level: {}", CONFIG_LOG_LEVEL);
    println!("Max CPUs: {}", CONFIG_MAX_CPUS);
    println!("Scheduler: {}", CONFIG_DEFAULT_SCHEDULER);
}
```

## 工作流程

```
1. 使用外部工具创建/编辑 .config → 如 make menuconfig 或手动创建
2. cargo-kbuild build                → 读取 .config 并编译项目
```

**简化的工作流程：**
- `.config` 由外部 Kconfig 工具管理
- cargo-kbuild 只负责读取和应用配置

## 与 Cargo 的区别

| 场景 | Cargo | cargo-kbuild |
|------|-------|--------------|
| 配置方式 | `--features` | `.config` 文件 |
| 依赖传递 | 树状传递 | 全局共享 |
| 配置来源 | 命令行 | 配置文件 |
| Feature 用途 | 功能开关 + 可选依赖 | 仅可选依赖 |
| 配置声明 | 必须在 Cargo.toml 中声明 | 可在代码中直接使用 |
| 适用场景 | 应用程序 | 内核/固件 |

## License

MIT OR Apache-2.0
