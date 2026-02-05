# cargo-kbuild 用户指南

cargo-kbuild 是一个让 Rust 项目能够使用 Kconfig 风格全局配置的构建工具。

## 安装

从源码安装：
```bash
git clone https://github.com/guoweikang/cargo-test.git
cd cargo-test
cargo install --path cargo-kbuild
```

验证安装：
```bash
cargo-kbuild --version
```

## 基本用法

### 1. 初始化项目

在 workspace 根目录运行：
```bash
cargo-kbuild init
```

这会：
- 扫描所有 `CONFIG_*` features
- 生成 `.config` 模板文件
- 更新 `.gitignore` 排除自动生成的文件

### 2. 配置项目

编辑生成的 `.config` 文件：
```bash
# 启用功能：将注释去掉
CONFIG_SMP=y
CONFIG_NET=y

# 禁用功能：设为 n 或注释掉
# CONFIG_DEBUG=n

# 数值配置
CONFIG_LOG_LEVEL=3
CONFIG_MAX_CPUS=8

# 字符串配置
CONFIG_DEFAULT_SCHEDULER="cfs"
```

### 3. 验证配置

在构建前检查配置有效性：
```bash
cargo-kbuild check
```

会检查：
- `.config` 文件语法
- 特性依赖关系
- 未使用的配置项
- 未定义的特性

### 4. 构建项目

使用默认 `.config` 构建：
```bash
cargo-kbuild build
```

使用自定义配置文件：
```bash
cargo-kbuild build --kconfig custom.config
```

## 进阶使用

### 在 Crate 中启用 kbuild

有两种方式启用 kbuild 支持：

**方式 1：显式启用（推荐）**
```toml
[package]
name = "my-kernel-module"

[package.metadata.kbuild]
enabled = true

[features]
CONFIG_SMP = []
CONFIG_PREEMPT = []
```

**方式 2：隐式启用**

只要定义 `CONFIG_*` 开头的 features，就会自动启用：
```toml
[features]
CONFIG_NET = []
CONFIG_ASYNC = []
```

### 使用配置值

#### 布尔配置

```rust
#[cfg(CONFIG_SMP)]
fn init_smp_subsystem() {
    println!("Initializing SMP");
}

#[cfg(not(CONFIG_SMP))]
fn init_up_subsystem() {
    println!("Initializing UP");
}
```

#### 数值和字符串配置

首先，添加 `kbuild_config` 依赖：
```toml
[dependencies]
kbuild_config = { path = "../kbuild_config" }
```

然后在代码中使用：
```rust
use kbuild_config::*;

fn init_logging() {
    println!("Log level: {}", CONFIG_LOG_LEVEL);
}

fn init_scheduler() {
    println!("Using {} scheduler", CONFIG_DEFAULT_SCHEDULER);
}
```

### 依赖关系规则

cargo-kbuild 对特性依赖有严格的验证规则：

#### ✅ 允许的依赖方式

1. **依赖支持 kbuild 的 crate（不指定子特性）**
```toml
[features]
CONFIG_NET = ["network_utils"]  # ✅ 正确
```

2. **依赖第三方库（可以指定子特性）**
```toml
[features]
CONFIG_LOGGING = ["log/std"]    # ✅ 正确
CONFIG_ASYNC = ["tokio/rt"]     # ✅ 正确
```

3. **依赖不支持 kbuild 的内部 crate（可以指定子特性）**
```toml
[features]
CONFIG_LEGACY = ["legacy_driver/usb"]  # ✅ 正确
```

#### ❌ 禁止的依赖方式

不能为支持 kbuild 的依赖指定子特性：
```toml
[features]
# ❌ 错误！network_utils 支持 kbuild
CONFIG_NET = ["network_utils/async"]
```

原因：支持 kbuild 的 crate 应该自己从 `.config` 读取配置，而不是由父 crate 控制。

### 配置文件格式

`.config` 文件使用简单的键值对格式：

```bash
# 注释以 # 开头

# 布尔值：y 表示启用，n 表示禁用
CONFIG_SMP=y
CONFIG_DEBUG=n

# 数值：整数
CONFIG_LOG_LEVEL=3
CONFIG_MAX_CPUS=8

# 字符串：用双引号包围
CONFIG_DEFAULT_SCHEDULER="cfs"
CONFIG_ARCH="x86_64"

# 禁用功能可以注释掉
# CONFIG_EXPERIMENTAL=y
```

### 自动生成的文件

cargo-kbuild 会生成以下文件，不应提交到 git：

1. **`.cargo/config.toml`**
   - 声明所有 `CONFIG_*` 选项给 rustc
   - 避免 "unexpected cfg" 警告
   - 每次 build 时自动重新生成

2. **`target/kbuild/config.rs`**
   - 包含数值和字符串配置的常量
   - 被 `kbuild_config` crate 包含

这些文件已在 `.gitignore` 中排除。

## 常见场景

### 场景 1：添加新功能

1. 在 crate 的 `Cargo.toml` 中添加 feature：
```toml
[features]
CONFIG_NEW_FEATURE = []
```

2. 运行 `cargo-kbuild check` 查看新功能：
```bash
cargo-kbuild check
```

3. 在 `.config` 中启用：
```bash
CONFIG_NEW_FEATURE=y
```

4. 构建：
```bash
cargo-kbuild build
```

### 场景 2：多个配置文件

为不同环境维护多个配置：

```bash
# 开发配置
cargo-kbuild build --kconfig .config.dev

# 生产配置
cargo-kbuild build --kconfig .config.prod

# 测试配置
cargo-kbuild build --kconfig .config.test
```

### 场景 3：调试配置问题

1. 检查语法和依赖：
```bash
cargo-kbuild check
```

2. 查看启用了哪些功能：
```bash
grep "=y" .config
```

3. 查看生成的 rustflags：
```bash
cat .cargo/config.toml
```

### 场景 4：迁移现有项目

1. 为需要全局配置的 features 添加 `CONFIG_` 前缀：
```toml
# 之前
[features]
smp = []

# 之后
[features]
CONFIG_SMP = []
```

2. 初始化 cargo-kbuild：
```bash
cargo-kbuild init
```

3. 编辑 `.config` 启用功能：
```bash
CONFIG_SMP=y
```

4. 更新代码中的 cfg 检查：
```rust
// 之前
#[cfg(feature = "smp")]

// 之后
#[cfg(CONFIG_SMP)]
```

5. 使用 cargo-kbuild 构建：
```bash
cargo-kbuild build
```

## 命令参考

### `cargo-kbuild init`

初始化项目配置。

**功能：**
- 扫描 workspace 中所有 `CONFIG_*` features
- 生成 `.config` 模板（如果不存在）
- 更新 `.gitignore`

**示例：**
```bash
cargo-kbuild init
```

### `cargo-kbuild check`

验证配置有效性。

**检查项：**
- `.config` 文件语法
- 特性依赖关系
- 未使用的配置项
- 未定义的特性

**示例：**
```bash
cargo-kbuild check
```

### `cargo-kbuild build`

构建项目。

**选项：**
- `--kconfig <path>`: 指定配置文件（默认：`.config`）

**示例：**
```bash
# 使用默认 .config
cargo-kbuild build

# 使用自定义配置文件
cargo-kbuild build --kconfig custom.config
```

### `cargo-kbuild --help`

显示帮助信息。

**示例：**
```bash
cargo-kbuild --help
```

### `cargo-kbuild --version`

显示版本信息。

**示例：**
```bash
cargo-kbuild --version
```

## 工作原理

### 构建流程

```
1. 解析 workspace
   ├─ 读取 Cargo.toml
   ├─ 扫描所有成员 crates
   └─ 收集 CONFIG_* features

2. 生成 .cargo/config.toml
   └─ 为每个 CONFIG_* 添加 --check-cfg

3. 验证特性依赖
   ├─ 检测 kbuild-enabled crates
   ├─ 验证依赖规则
   └─ 报告错误

4. 解析 .config
   ├─ 读取配置值
   └─ 生成 target/kbuild/config.rs

5. 构建项目
   ├─ 设置 RUSTFLAGS
   ├─ 传递 --cfg 标志
   └─ 调用 cargo build
```

### 智能检测

cargo-kbuild 通过两种方式检测 crate 是否支持 kbuild：

1. **显式声明：**
```toml
[package.metadata.kbuild]
enabled = true
```

2. **隐式检测：**
   - 有任何 `CONFIG_*` 开头的 features

检测到的 kbuild-enabled crates 受严格的依赖规则约束。

### 配置传递

与 Cargo 的 features 不同，kbuild 配置不通过依赖树传递：

```
Cargo features:        cargo-kbuild:

parent                 .config
  |                      |
  +-- child1            ├── crate1
  |                     ├── crate2
  +-- child2            └── crate3

树状传递               全局共享
```

所有 crate 平等地从 `.config` 读取配置。

## 故障排除

### 错误：找不到 .config

```
❌ Error: .config file not found
```

**解决：**
```bash
cargo-kbuild init
```

### 错误：不能为 kbuild-enabled 依赖指定子特性

```
❌ Error in crate 'my-crate':
Feature 'CONFIG_NET' specifies sub-feature: 'network_utils/async'
```

**解决：**
移除子特性，让依赖自己从 .config 读取：
```toml
# 之前
CONFIG_NET = ["network_utils/async"]

# 之后
CONFIG_NET = ["network_utils"]
```

### 警告：配置项未使用

```
⚠️ Warning: The following configs are defined in .config but not declared in any crate:
   - CONFIG_MY_FEATURE
```

**解决：**
1. 检查配置名是否拼写错误
2. 在某个 crate 的 Cargo.toml 中声明该 feature
3. 或从 .config 中移除

### rustc 警告：unexpected cfg

```
warning: unexpected `cfg` condition name: `CONFIG_XXX`
```

**解决：**
运行 `cargo-kbuild build` 而不是 `cargo build`，会自动生成 .cargo/config.toml。

## 最佳实践

1. **始终用 cargo-kbuild 命令**
   - 使用 `cargo-kbuild build` 而不是 `cargo build`
   - 确保 .cargo/config.toml 是最新的

2. **提交 .config 到 git**
   - 提供默认配置
   - 方便团队协作

3. **不提交自动生成的文件**
   - `.cargo/config.toml`
   - `target/kbuild/`

4. **使用 check 命令**
   - 在构建前运行 `cargo-kbuild check`
   - 及早发现配置问题

5. **规范命名**
   - 所有全局配置使用 `CONFIG_` 前缀
   - 使用大写和下划线：`CONFIG_MY_FEATURE`

6. **文档化配置**
   - 在 .config 中添加注释说明各配置的用途
   - 在 README 中列出可用配置

## 示例项目

本仓库的 `crates/` 目录包含完整示例：

- **kernel_task** - 基本 kbuild 使用
- **kernel_schedule** - 依赖其他 kbuild crates
- **kernel_net** - 多个 CONFIG_* features
- **demo_mixed_deps** - 混合使用 kbuild crates 和第三方库

查看它们的 `Cargo.toml` 学习如何配置。

## 技术细节

如需了解实现细节，请参阅：
- [IMPLEMENTATION_DETAILS.md](../IMPLEMENTATION_DETAILS.md) - 详细技术架构
- [IMPLEMENTATION_SUMMARY.md](../IMPLEMENTATION_SUMMARY.md) - 功能总结

## 反馈

遇到问题或有建议？请在 GitHub 提 issue。
