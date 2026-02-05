# cargo-kbuild

在 Rust 项目中使用 Kconfig 风格的全局配置。

## 为什么需要这个工具？

Cargo 的 feature 是树状传递的：父包开启 feature，子包被动接收。

但内核开发不是这样的：所有模块平等地读取同一份配置文件。

cargo-kbuild 实现了这种"全局配置"模式。

## 快速开始

安装：
```bash
cargo install --path cargo-kbuild
```

初始化项目：
```bash
cd your-project
cargo-kbuild init
```

编辑 `.config` 文件：
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

1. 扫描项目中所有 `CONFIG_*` features
2. 根据 `.config` 文件设置编译参数
3. 所有 crate 看到相同的配置

不需要写 build.rs，不需要手动管理 cfg 声明。

## 示例

参考 `crates/` 目录中的示例 crates：
- `kernel_task/` - 基本的 CONFIG_* 使用
- `kernel_net/` - 依赖其他 kbuild crates
- `demo_mixed_deps/` - 混合使用第三方库

## 命令

```bash
cargo-kbuild init      # 初始化项目，生成 .config 模板
cargo-kbuild check     # 检查配置有效性
cargo-kbuild build     # 编译项目
cargo-kbuild --help    # 显示帮助
cargo-kbuild --version # 显示版本
```

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

### 全局配置

所有启用 kbuild 的 crate 都从同一个 `.config` 读取配置：

```rust
// crates/kernel_task/Cargo.toml
[features]
CONFIG_SMP = []
CONFIG_PREEMPT = []

// crates/kernel_net/Cargo.toml
[features]
CONFIG_NET = []
CONFIG_ASYNC = []
```

一个 `.config` 文件控制所有：
```
CONFIG_SMP=y
CONFIG_NET=y
```

### 自动化

- 自动生成 `.cargo/config.toml` 声明所有 CONFIG_*
- 自动验证特性依赖关系
- 自动检测配置错误

## 启用 kbuild

在 crate 的 `Cargo.toml` 中：

```toml
[package.metadata.kbuild]
enabled = true

[features]
CONFIG_SMP = []
CONFIG_PREEMPT = []
```

或者只要定义 `CONFIG_*` features 就会自动启用。

## 使用配置值

### 布尔配置

```rust
#[cfg(CONFIG_SMP)]
fn init_smp() {
    println!("SMP enabled");
}
```

### 数值和字符串配置

在 `.config` 中：
```
CONFIG_LOG_LEVEL=3
CONFIG_DEFAULT_SCHEDULER="cfs"
```

在代码中：
```rust
// 需要 crate 依赖 kbuild_config
use kbuild_config::*;

fn init() {
    println!("Log level: {}", CONFIG_LOG_LEVEL);
    println!("Scheduler: {}", CONFIG_DEFAULT_SCHEDULER);
}
```

## 工作流程

```
1. cargo-kbuild init    → 生成 .config 模板
2. 编辑 .config          → 启用需要的功能
3. cargo-kbuild check   → 验证配置
4. cargo-kbuild build   → 编译项目
```

## 与 Cargo 的区别

| 场景 | Cargo | cargo-kbuild |
|------|-------|--------------|
| 配置方式 | `--features` | `.config` 文件 |
| 依赖传递 | 树状传递 | 全局共享 |
| 配置来源 | 命令行 | 配置文件 |
| 适用场景 | 应用程序 | 内核/固件 |

## License

MIT OR Apache-2.0
