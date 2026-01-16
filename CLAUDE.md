# CLAUDE.md

## 项目概述

**Czkawka** (波兰语"打嗝") 是一个用 Rust 编写的开源文件清理工具，用于查找和删除不必要的文件（重复文件、空文件夹、相似图片等）。

### 项目结构

| 模块 | 描述 | 框架 |
|------|------|------|
| `czkawka_core` | 核心库，包含所有扫描逻辑 | 纯 Rust |
| `czkawka_gui` | GTK 4 图形界面 | GTK 4 |
| `czkawka_cli` | 命令行界面 | clap |
| `krokiet` | Slint 图形界面 | Slint |

### 支持的工具/功能

- **Duplicates** - 基于文件名、大小或哈希查找重复文件
- **Empty Folders/Files** - 查找空文件夹和空文件
- **Big Files** - 查找最大的文件
- **Similar Images** - 查找相似图片（不同分辨率、水印）
- **Similar Videos** - 查找视觉上相似的视频
- **Same Music** - 通过标签或内容查找相似音乐
- **Invalid Symlinks** - 显示无效的符号链接
- **Broken Files** - 查找损坏的文件
- **Bad Extensions** - 列出扩展名与内容不匹配的文件
- **Temporary Files** - 查找临时文件

## 开发命令

### 构建

```bash
# 构建所有模块
cargo build --release

# 构建特定模块
cargo build --release --bin czkawka_gui
cargo build --release --bin czkawka_cli
cargo build --release --bin krokiet
```

### 运行

```bash
# 使用 justfile（推荐）
just run czkawka_gui
just run czkawka_cli
just run krokiet

# 或直接使用 cargo
cargo run --bin czkawka_gui
cargo run --bin czkawka_cli
cargo run --bin krokiet
```

### 测试和检查

```bash
# 运行所有测试
cargo test

# 运行 clippy
cargo clippy --all-features --all-targets

# 运行集成测试
just itests

# 运行基准测试
just bench
```

### 代码格式化和修复

```bash
# 完整的修复流程（格式化 + clippy 修复 + 翻译检查）
just fix

# 仅格式化
cargo fmt
cargo +nightly fmt
```

## 代码规范

### Clippy 规则

项目使用严格的 clippy 规则，定义在 `Cargo.toml` 的 `[workspace.lints]` 中：

- `unwrap_used = "warn"` - 避免使用 `.unwrap()`
- `indexing_slicing = "warn"` - 避免直接索引
- `print_stdout/print_stderr = "warn"` - 避免直接打印
- `todo/unimplemented = "warn"` - 避免 TODO 标记
- 启用了大量其他 lint 规则

### 编译配置

- **Rust 版本**: 1.90.0+, Edition 2024
- **panic**: `unwind`（允许捕获 panic）
- **overflow-checks**: 启用（发现隐藏的 panic）
- **dev 依赖优化**: opt-level = 3（即使在 debug 模式下也优化依赖）

### Profile 说明

| Profile | 用途 |
|---------|------|
| `release` | 标准发布构建 |
| `fast_release` | 增量发布构建，开发时使用 |
| `fastci` | CI 快速构建，小二进制 |
| `rdebug` | 带调试信息的发布构建 |
| `fastest` | 最小最快的二进制（unsafe） |

## 核心模块结构 (czkawka_core)

```
czkawka_core/src/
├── lib.rs              # 公共 API 导出
├── localizer_core.rs   # 国际化
├── common/             # 通用功能
│   ├── dir_traversal.rs    # 目录遍历
│   ├── cache.rs            # 缓存处理
│   ├── extensions.rs       # 文件扩展名
│   └── ...
├── helpers/            # 辅助函数
└── tools/              # 各个扫描工具
    ├── duplicate/
    ├── similar_images/
    ├── similar_videos/
    ├── same_music/
    ├── broken_files/
    ├── empty_folder/
    ├── empty_files/
    ├── big_file/
    ├── temporary/
    ├── invalid_symlinks/
    └── bad_extensions/
```

## 国际化 (i18n)

项目使用 Fluent 进行国际化，支持多种语言。

翻译文件位置：
- `czkawka_core/i18n/`
- `czkawka_gui/i18n/`
- `krokiet/i18n/`

检查未使用的翻译：
```bash
python3 misc/find_unused_fluent_translations.py czkawka_gui
python3 misc/find_unused_fluent_translations.py krokiet
python3 misc/find_unused_fluent_translations.py czkawka_core
```

## 缓存和配置

配置文件位置：
- Linux: `~/.config/czkawka`
- Windows: `%APPDATA%\Qarmin\Czkawka\config`
- macOS: `~/Library/Application Support/pl.Qarmin.Czkawka`

缓存文件位置：
- Linux: `~/.cache/czkawka`
- Windows: `%LOCALAPPDATA%\Qarmin\Czkawka\cache`
- macOS: `~/Library/Caches/pl.Qarmin.Czkawka`

可通过环境变量覆盖：`CZKAWKA_CONFIG_PATH`, `CZKAWKA_CACHE_PATH`

## 依赖关系

### 主要依赖

- **image** - 图像处理
- **image_hasher** - 图像哈希（相似图片检测）
- **vid_dup_finder_lib** - 视频重复检测
- **lofty** - 音频元数据
- **rusty-chromaprint** - 音频指纹
- **blake3/crc32fast/xxhash** - 文件哈希
- **rayon** - 并行处理
- **serde/bincode** - 序列化/缓存

### GUI 依赖

- **czkawka_gui**: GTK 4 (gtk4-rs)
- **krokiet**: Slint

## 注意事项

1. **几乎无 unsafe 代码** - 项目追求内存安全
2. **多线程** - 大量使用 rayon 进行并行处理
3. **缓存支持** - 第二次及后续扫描会更快
4. **无网络访问** - 不收集用户信息
