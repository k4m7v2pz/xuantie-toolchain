// 玄铁工具链入口 — cargo 风格 subcommand,转发给底层 Go 编译器
use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;

mod commands;
mod lock;
mod manifest;

const HELP: &str = r#"玄铁 (XuanTie) 中文编程语言工具链

用法:
    xuantie <命令> [选项] [参数]

命令:
    new <名称>         创建新玄铁项目
    init               在当前目录初始化玄铁项目
    add <库名> <url>   添加依赖并拉取 (--tag/--branch/--rev/--path)
    run [文件]         解释执行 .xt 文件 (默认入口:主函数.xt)
    run --watch        文件变动自动重跑,直到 Ctrl+C
    check [文件]       语法检查不执行 (CI 用)
    build [文件]       编译为独立可执行文件到 dist/
    build --tie        用 LLVM 编译为原生二进制 (需 clang/gcc)
    fmt                格式化代码 (玄铁暂无格式化器,此命令预留)
    test               运行 tests/ 下所有 .xt 测试,按 exit code 判通过/失败
    lint [文件]        静态分析:符号摘要 + 重复定义/导入检测
    clean [--all]      清理 dist/ 和可执行产物 (--all 含 tiepm_modules/)
    version            显示工具链版本
    help               显示本帮助

选项:
    -V, --version      等同 `xuantie version`
    -h, --help         等同 `xuantie help`

示例:
    xuantie new 我的项目
    cd 我的项目
    xuantie run
    xuantie build
    xuantie test

环境:
    底层编译器由 build.rs 编译,路径在编译期注入。
    build --tie 需要 clang 和 gcc/MinGW;run/check/test 仅需 CLI 自身已编译。

工具链 CLI 公有领域 (Unlicense);玄铁语言实现 MIT,归原作者问号盒。
详见 NOTICE 与 LICENSE.upstream.MIT。
"#;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print!("{}", HELP);
        return Ok(());
    }
    let cmd = args[1].as_str();
    let rest = &args[2..];

    match cmd {
        "-h" | "--help" | "help" => {
            print!("{}", HELP);
            Ok(())
        }
        "-V" | "--version" | "version" => commands::version::version(rest),
        "new" => commands::new_cmd::create(rest),
        "init" => commands::new_cmd::init(rest),
        "add" => commands::add::add(rest),
        "run" => commands::run::run(rest),
        "check" => commands::check::check(rest),
        "build" => commands::build::build(rest),
        "fmt" => commands::fmt::fmt(rest),
        "test" => commands::test::test(rest),
        "lint" => commands::lint::lint(rest),
        "clean" => commands::clean::clean(rest),
        other => Err(anyhow!("未知命令: {}\n用 `xuantie help` 查看可用命令", other)),
    }
}

/// 找到底层玄铁编译器二进制路径
/// build.rs 通过 env! 注入;若为空说明编译失败
pub(crate) fn core_path() -> Result<PathBuf> {
    let p = env!("XUANTIE_CORE_PATH");
    if p.is_empty() {
        return Err(anyhow!(
            "底层玄铁编译器未编译 (build.rs 未找到 go)\n\
             请安装 Go 后重新 cargo build"
        ));
    }
    Ok(PathBuf::from(p))
}
