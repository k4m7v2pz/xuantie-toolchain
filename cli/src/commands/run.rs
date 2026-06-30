// xuantie run [文件] [--watch]
// 默认跑 xuantie.toml 里 entry 指定的入口,无 toml 时跑 主函数.xt
// --watch/-w:文件变动后自动重跑,直到 Ctrl+C
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

pub fn run(args: &[String]) -> Result<()> {
    let watch = args.iter().any(|a| a == "--watch" || a == "-w");
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

    if watch {
        run_watch(&core, &entry)
    } else {
        run_once(&core, &entry)
    }
}

/// 单次执行,转发输出,失败 exit 非 0
fn run_once(core: &PathBuf, entry: &PathBuf) -> Result<()> {
    // 玄铁编译器:`xuantie <源文件>` 直接解释执行
    let out = Command::new(core)
        .arg(entry)
        .output()
        .map_err(|e| anyhow!("启动解释器失败: {}", e))?;

    use std::io::Write;
    std::io::stdout().write_all(&out.stdout).ok();
    std::io::stderr().write_all(&out.stderr).ok();
    let code = out.status.code().unwrap_or(-1);
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

/// watch 模式:轮询监听 .xt 文件 mtime,变动就重跑
fn run_watch(core: &PathBuf, entry: &PathBuf) -> Result<()> {
    println!("👁 watch 模式已启动 (Ctrl+C 退出),监听 .xt 文件变动...");
    println!("\n{}", "─".repeat(40));
    run_once(core, entry)?;

    let mut last = snapshot_mtimes();
    loop {
        std::thread::sleep(Duration::from_secs(1));
        let now = snapshot_mtimes();
        if now != last {
            last = now;
            println!("\n{}\n⚡ 检测到文件变动,重跑...\n{}", "─".repeat(40), "─".repeat(40));
            if let Ok(m) = crate::manifest::load_current() {
                let (ready, issues) = crate::commands::add::ensure_deps_ready(&m);
                if !ready {
                    eprintln!("⚠ 依赖未就绪,跳过本次执行:");
                    for i in &issues {
                        eprintln!("  - {}", i);
                    }
                    continue;
                }
            }
            let out = match Command::new(core).arg(entry).output() {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("⚠ 启动解释器失败: {}", e);
                    continue;
                }
            };
            use std::io::Write;
            std::io::stdout().write_all(&out.stdout).ok();
            std::io::stderr().write_all(&out.stderr).ok();
            if !out.status.success() {
                eprintln!("⚠ 本次执行失败 (exit {}),继续监听...", out.status.code().unwrap_or(-1));
            }
        }
    }
}

/// 收集项目所有 .xt 文件的 (路径, mtime) 快照
fn snapshot_mtimes() -> Vec<(PathBuf, std::time::SystemTime)> {
    let mut snap = Vec::new();
    let collect_dir = |dir: &std::path::Path, snap: &mut Vec<(PathBuf, std::time::SystemTime)>| {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if matches!(name, "dist" | "target" | ".git" | "node_modules" | "tiepm_modules") {
                    continue;
                }
                // 递归一层 (玄铁项目规模通常不大)
                if let Ok(sub) = std::fs::read_dir(&p) {
                    for se in sub.flatten() {
                        let sp = se.path();
                        if sp.extension().map_or(false, |x| x == "xt") {
                            if let Ok(m) = std::fs::metadata(&sp).and_then(|m| m.modified()) {
                                snap.push((sp, m));
                            }
                        }
                    }
                }
            } else if p.extension().map_or(false, |x| x == "xt") {
                if let Ok(m) = std::fs::metadata(&p).and_then(|m| m.modified()) {
                    snap.push((p, m));
                }
            }
        }
    };

    collect_dir(std::path::Path::new("."), &mut snap);
    collect_dir(std::path::Path::new("src"), &mut snap);
    collect_dir(std::path::Path::new("tiepm_modules"), &mut snap);
    snap.sort_by(|a, b| a.0.cmp(&b.0));
    snap
}
