// xuantie clean [--all]
// 清理构建产物:dist/ 和散落的可执行/.ll/.o
// 不删 tiepm_modules/ (那是依赖,删了得重新拉);要删依赖用 xuantie clean --all
use anyhow::{anyhow, Result};
use std::path::Path;

pub fn clean(args: &[String]) -> Result<()> {
    let purge_all = args.iter().any(|a| a == "--all" || a == "-a");
    let mut removed: Vec<String> = Vec::new();

    if Path::new("dist").is_dir() {
        std::fs::remove_dir_all("dist")
            .map_err(|e| anyhow!("删除 dist/ 失败: {}", e))?;
        removed.push("dist/".into());
    }

    // 散落产物 (玄铁 build/tie 会生成 exe/.ll/.o)
    for entry in std::fs::read_dir(".").map_err(|e| anyhow!("读取当前目录失败: {}", e))? {
        let p = entry?.path();
        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            if matches!(ext, "ll" | "o" | "chex") {
                if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                    std::fs::remove_file(&p).ok();
                    removed.push(name.into());
                }
            }
        }
        // 顶层裸 exe (windows);非 windows 顶层裸产物较罕见,忽略
        if cfg!(target_os = "windows") {
            if p.extension().map_or(false, |e| e == "exe") {
                if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                    if name != "xuantie.exe" && name != "xuantie_core.exe" {
                        std::fs::remove_file(&p).ok();
                        removed.push(name.into());
                    }
                }
            }
        }
    }

    if purge_all && Path::new("tiepm_modules").is_dir() {
        std::fs::remove_dir_all("tiepm_modules")
            .map_err(|e| anyhow!("删除 tiepm_modules/ 失败: {}", e))?;
        removed.push("tiepm_modules/".into());
    }

    if removed.is_empty() {
        println!("无需清理 (没有产物)");
    } else {
        println!("✓ 已清理:");
        for r in &removed {
            println!("  {}", r);
        }
    }
    Ok(())
}
