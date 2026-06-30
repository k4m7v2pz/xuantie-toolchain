# 玄铁工具链 (xuantie-toolchain)

中文编程语言 **玄铁 (XuanTie)** 的命令行工具链,cargo/uv 风格。提供项目脚手架、运行、编译、检查、测试、依赖管理等命令,内置玄铁编译器 (Go 实现的种子编译器,fork 自上游 [XuanTie-Lang](https://github.com/MARKJY-China/XuanTie-Lang))。

**先看 [立场声明](STATEMENT.md)** — 我们要做开源、免费、跨平台、git 友好的工具链,把玄铁包裹成 cargo/uv 那样的体验。

本仓库只做命令行工具链,**不含图形界面**。IDE/编辑器是另一件事,后续以 VSCode 插件等形式独立解决 (上游已有 `extensions/xuantie-syntax` 语法高亮扩展可作起点)。

> **许可**:工具链 CLI 部分 (`cli/` 目录) 采用 **Unlicense** (公有领域),可任意使用。
> 玄铁语言实现 (编译器、运行时、标准库等,即 `cli/` 之外的全部源码) 继承自上游,按 **MIT** 许可,归原作者 **问号盒** 所有。详见 [NOTICE](NOTICE) 与 [LICENSE.upstream.MIT](LICENSE.upstream.MIT)。

> **上游文档**:玄铁语言本身的介绍、语法指南、设计哲学见上游原有文档 [readme.md](readme.md) / [README_EN.md](README_EN.md) / [GUIDE/](GUIDE/) / [玄铁的定位.md](玄铁的定位.md)。

---

## 安装

需要系统已安装 **Go** (编译底层玄铁种子编译器) 和 **Rust/Cargo** (编译本 CLI)。

```bash
git clone git@github.com:k4m7v2pz/xuantie-toolchain.git
cd xuantie-toolchain/cli
cargo install --path .
```

`cargo build` 时 `build.rs` 会自动用 Go 编译底层玄铁编译器,产物路径在编译期注入到 Rust 端,运行时直接定位,免去查找开销。

装好后 `xuantie` 命令可用:

```bash
xuantie version   # xuantie 0.1.0
xuantie help
```

---

## 命令

```
xuantie new <项目名>      # 创建新项目 (主函数.xt + xuantie.toml + src/ + tests/)
xuantie init              # 在当前目录初始化项目
xuantie add <库名> <url>  # 添加依赖并拉取 (--tag/--branch/--rev/--path)
xuantie run [文件]        # 解释执行入口文件 (默认 xuantie.toml 的 entry 或 主函数.xt)
xuantie run --watch       # 文件变动自动重跑,直到 Ctrl+C
xuantie check [文件]      # 语法检查不执行 (CI 用)
xuantie build [文件]      # 编译为独立可执行文件到 dist/
xuantie build --tie       # 用 LLVM 编译为原生二进制 (需 clang/gcc)
xuantie fmt [--check]     # 格式化 (玄铁暂无格式化器,此命令预留)
xuantie test              # 跑 tests/*.xt,按 exit code 判通过/失败
xuantie lint [文件]       # 静态分析:符号摘要 + 重复定义/导入检测
xuantie clean [--all]     # 清理 dist/ 和产物 (--all 含 tiepm_modules/)
xuantie version           # 显示工具链版本
xuantie help              # 显示帮助
```

---

## 项目结构 (`xuantie new` 生成)

```
我的项目/
├── 主函数.xt           入口文件
├── xuantie.toml        项目配置
├── src/                源码目录
├── tests/              测试目录
└── .gitignore
```

`xuantie.toml` 示例:

```toml
[package]
name = "我的项目"
version = "0.1.0"
entry = "主函数.xt"

[profile.dev]
opt_level = 0
debug = true

[profile.release]
opt_level = 2
debug = false

# 依赖三种形式:
#   库名 = { git = "url", tag = "v0.1.0" }
#   库名 = { git = "url", branch = "main" }
#   库名 = { path = "../本地库" }
[dependencies]
```

---

## 依赖管理

`xuantie add` 把依赖写进 `xuantie.toml` 并拉到 `tiepm_modules/`,同时更新 `xuantie.lock` 锁定 commit hash,保证可复现构建。

```bash
xuantie add 数组工具 https://github.com/xxx/array-utils.git --tag v0.1.0
xuantie add 工具库 https://github.com/xxx/lib.git --branch main
xuantie add 本地库 --path ../my-lib
```

`run`/`build`/`check` 执行前会校验依赖齐全且与 lock 一致,缺失或漂移会给出修复提示。

> 依赖目录沿用玄铁生态的 **铁铺 (TiePM)** 命名 (`tiepm_modules/`),与上游玄铁包管理器概念对齐。

---

## 架构

```
xuantie-toolchain/
├── main.go              玄铁种子编译器入口 (Go, fork 自上游)
├── lexer/ parser/ ast/  词法 / 语法 / AST
├── compiler/            Go 转译器 + LLVM 后端
├── evaluator/           解释器
├── object/ token/ stdlib/ runtime/
├── lib/ 渲染/           玄铁标准库与渲染桥
├── tiepm/               铁铺包管理器 (玄铁自举实现,上游)
├── Test/ examples/ GUIDE/  上游测试、示例、文档
└── cli/                 Rust wrapper (cargo 风格入口,本仓库新增)
    ├── Cargo.toml
    ├── build.rs         编译期用 Go 编出底层编译器,注入路径
    └── src/commands/    new / run / build / check / test / lint / add / clean / fmt / version
```

CLI 是 wrapper,不修改玄铁语言实现的核心逻辑,通过子进程调用底层编译器。`cargo build` 时 `build.rs` 会自动用 Go 编译底层种子编译器,产物路径在编译期注入到 Rust 端。

---

## 已知限制

- **`build` 子命令**:底层玄铁 Go 转译器 (`compiler/compiler.go`) 生成的 Go 代码含 Windows FFI syscall,在非 Windows 平台编译会失败。这是上游玄铁的既有限制 (原作者主要在 Windows 开发),本工具链未修改转译器逻辑。**Windows 上 `build` 正常**,非 Windows 暂请用 `xuantie run` 解释执行。
- **`build --tie`**:LLVM 原生编译路径同样含 Windows 目标硬编码 (`-target x86_64-w64-windows-gnu`、`C:/TDM-GCC-64` 等),非 Windows 暂不可用。
- **`fmt`**:玄铁尚无官方格式化器。上游玄铁在 v1.0 自举定型前语法可能变动,格式化器推迟到那时再开发。本命令预留,`--check` 跳过通过,实际格式化会给出说明。
- 玄铁语言本身仍处于 0.x 阶段,语法可能变动。工具链追随上游。

---

## 与上游的关系

本仓库 fork 自 [MARKJY-China/XuanTie-Lang](https://github.com/MARKJY-China/XuanTie-Lang),上游仓库内容完整保留 (编译器、运行时、标准库、铁铺、测试、指南等),只在根目录新增了 `cli/` 工具链与许可/声明文件,以及为跨平台编译做的必要隔离 (Windows FFI 与 console API 的 build tag 分流)。

- 上游玄铁语言实现的著作权归原作者 **问号盒**,按 MIT 许可;
- 工具链 CLI (`cli/`) 是本仓库在公有领域 (Unlicense) 下贡献的新代码。

上游原有的 `readme.md`、`GUIDE/`、`Test/`、`examples/` 等文档与测试保留不动,作为玄铁语言本身的文档。

---

## 许可证

- 工具链 CLI (`cli/`): **Unlicense** (公有领域),见 [LICENSE](LICENSE)
- 玄铁语言实现 (其余源码): **MIT**,归原作者问号盒,见 [LICENSE.upstream.MIT](LICENSE.upstream.MIT)
- 归属声明: [NOTICE](NOTICE)
