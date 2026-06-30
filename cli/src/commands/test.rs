// xuantie test
// 跑 tests/ 下所有 .xt 文件,汇总通过/失败
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;

pub fn test(_args: &[String]) -> Result<()> {
    let core = crate::core_path()?;
    let tests_dir = std::path::Path::new("tests");
    if !tests_dir.exists() {
        return Err(anyhow!("未找到 tests/ 目录\n用 `xuantie new` 创建的项目会自带 tests/"));
    }

    let mut tests: Vec<PathBuf> = std::fs::read_dir(tests_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "xt"))
        .collect();
    tests.sort();

    if tests.is_empty() {
        println!("tests/ 目录为空,无测试可跑");
        return Ok(());
    }

    let mut passed = 0;
    let mut failed = 0;
    println!("运行 {} 个测试...", tests.len());
    println!("{}", "─".repeat(40));

    for t in &tests {
        let name = t.file_name().and_then(|s| s.to_str()).unwrap_or("?");
        let out = Command::new(&core).arg(t).output()?;

        let status = out.status.success();
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);

        // 简单约定:exit 0 视为通过,非 0 视为失败
        if status {
            println!("✓ {}  ({})", name, stdout.trim());
            passed += 1;
        } else {
            println!("✗ {}  (exit {})", name, out.status.code().unwrap_or(-1));
            if !stderr.is_empty() {
                println!("    {}", stderr.trim());
            }
            failed += 1;
        }
    }

    println!("{}", "─".repeat(40));
    println!("结果: {} 通过, {} 失败", passed, failed);
    if failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}
