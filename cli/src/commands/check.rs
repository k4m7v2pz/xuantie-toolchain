// xuantie check [文件]
// 语法/运行时检查:调底层跑一遍,丢 stdout,只在 stderr 非空或 exit 非 0 时报错
use anyhow::{anyhow, Result};
use std::process::Command;

pub fn check(args: &[String]) -> Result<()> {
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

    let out = Command::new(&core)
        .arg(&entry)
        .output()
        .map_err(|e| anyhow!("启动解释器失败: {}", e))?;

    let code = out.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&out.stderr);

    if code != 0 || !stderr.trim().is_empty() {
        use std::io::Write;
        std::io::stderr().write_all(&out.stderr).ok();
        return Err(anyhow!("检查失败 (exit {}): {}", code, entry.display()));
    }

    println!("✓ {} 语法检查通过", entry.display());
    Ok(())
}
