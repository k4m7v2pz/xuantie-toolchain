// xuantie build [文件] [--tie]
// 编译为独立可执行文件,输出到 dist/
//   默认:用玄铁 build 子命令 (Go 转译路径,跨平台纯静态)
//   --tie:用玄铁 tie 子命令 (LLVM 原生编译,需 clang/gcc)
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;

pub fn build(args: &[String]) -> Result<()> {
    let use_tie = args.iter().any(|a| a == "--tie" || a == "-t");
    let manifest = crate::manifest::load_current().ok();
    if let Some(m) = &manifest {
        let (ready, issues) = crate::commands::add::ensure_deps_ready(m);
        if !ready {
            let mut msg = String::from("依赖未就绪:\n");
            for i in &issues {
                msg.push_str(&format!("  - {}\n", i));
            }
            msg.push_str("修复后重试");
            return Err(anyhow!("{}", msg));
        }
    }
    let entry = crate::manifest::resolve_entry(manifest.as_ref(), args)?;
    let core = crate::core_path()?;

    std::fs::create_dir_all("dist")?;
    let stem = entry
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let exe_name = if cfg!(target_os = "windows") {
        format!("{}.exe", stem)
    } else {
        stem.to_string()
    };
    let out_path = PathBuf::from("dist").join(&exe_name);

    // 玄铁编译器:
    //   xuantie build <源文件> --out <产物>    (Go 转译)
    //   xuantie tie   <源文件> --out <产物>    (LLVM 原生)
    let sub = if use_tie { "tie" } else { "build" };
    let out = Command::new(&core)
        .arg(sub)
        .arg(&entry)
        .arg("--out")
        .arg(&out_path)
        .output()
        .map_err(|e| anyhow!("启动编译器失败: {}", e))?;

    use std::io::Write;
    std::io::stderr().write_all(&out.stderr).ok();
    if !out.status.success() {
        return Err(anyhow!(
            "编译失败 (exit {})",
            out.status.code().unwrap_or(-1)
        ));
    }
    println!("✓ 已编译: {} → {}", entry.display(), out_path.display());
    println!("  运行: ./{}", out_path.display());
    Ok(())
}
