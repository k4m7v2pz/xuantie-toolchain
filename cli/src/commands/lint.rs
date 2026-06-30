// xuantie lint [文件]
// 轻量静态检查:符号摘要 + 基础问题检测
// 纯文本分析,不调底层编译器,快且不依赖编译器
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::path::PathBuf;

pub fn lint(args: &[String]) -> Result<()> {
    let manifest = crate::manifest::load_current().ok();
    let entry = crate::manifest::resolve_entry(manifest.as_ref(), args)?;
    // lint 不调底层,但确认底层存在 (保持一致的环境感知)
    let _ = crate::core_path()?;

    let files: Vec<PathBuf> = if args.iter().any(|a| !a.starts_with('-')) {
        vec![entry]
    } else {
        collect_lint_files()?
    };

    if files.is_empty() {
        return Err(anyhow!("未找到要 lint 的 .xt 文件"));
    }

    let mut total_issues = 0;
    for f in &files {
        total_issues += lint_one(f)?;
    }

    if total_issues == 0 {
        println!("\n✓ lint 通过 ({} 个文件)", files.len());
    } else {
        eprintln!("\n✗ lint 发现 {} 个问题", total_issues);
        std::process::exit(1);
    }
    Ok(())
}

fn lint_one(path: &PathBuf) -> Result<usize> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| anyhow!("读取 {} 失败: {}", path.display(), e))?;

    println!("\n📄 {}", path.display());

    let mut functions: Vec<(String, usize)> = Vec::new();
    let mut classes: Vec<(String, usize)> = Vec::new();
    let mut imports: Vec<(String, usize)> = Vec::new();
    let mut globals: Vec<(String, usize)> = Vec::new();

    // 玄铁关键字:函/类/引/设
    for (i, line) in src.lines().enumerate() {
        let n = i + 1;
        let t = line.trim();
        if t.is_empty() || t.starts_with("//") || t.starts_with("#") {
            continue;
        }
        // 引 "路径" / 引 "路径" 予 别名
        if let Some(p) = extract_import(t) {
            imports.push((p, n));
            continue;
        }
        // 类 名称 { / 类 名称(
        if let Some(name) = extract_class(t) {
            classes.push((name, n));
            continue;
        }
        // 函 名称(
        if let Some(name) = extract_fn(t) {
            functions.push((name, n));
            continue;
        }
        // 设 名称 = (顶层变量,粗略判断)
        if let Some(name) = extract_let(t) {
            globals.push((name, n));
        }
    }

    println!("  函数 {} 个, 类 {} 个, 导入 {} 个, 全局变量 {} 个",
        functions.len(), classes.len(), imports.len(), globals.len());
    for (name, line) in &functions {
        println!("    函 {}() @{}", name, line);
    }
    for (name, line) in &classes {
        println!("    类 {} @{}", name, line);
    }
    for (p, line) in &imports {
        println!("    引 \"{}\" @{}", p, line);
    }

    let mut issues = 0;
    let mut seen: HashSet<&str> = HashSet::new();

    // 重复函数
    for (name, line) in &functions {
        if !seen.insert(name.as_str()) {
            println!("  ⚠ 重复函数定义: {} @{}", name, line);
            issues += 1;
        }
    }
    // 重复类
    seen.clear();
    for (name, line) in &classes {
        if !seen.insert(name.as_str()) {
            println!("  ⚠ 重复类定义: {} @{}", name, line);
            issues += 1;
        }
    }
    // 重复导入
    seen.clear();
    for (p, line) in &imports {
        if !seen.insert(p.as_str()) {
            println!("  ⚠ 重复导入: \"{}\" @{}", p, line);
            issues += 1;
        }
    }
    // 空文件
    if functions.is_empty() && classes.is_empty() && globals.is_empty() && imports.is_empty() {
        println!("  ⚠ 文件无任何定义 (空文件或仅含注释)");
        issues += 1;
    }

    Ok(issues)
}

/// 提取 `引 "路径"` 的路径
fn extract_import(line: &str) -> Option<String> {
    let t = line.trim_start();
    if !t.starts_with("引") && !t.starts_with("import") {
        return None;
    }
    let kw_end = if t.starts_with("引") { "引".len() } else { "import".len() };
    let after = t[kw_end..].trim_start();
    // 找引号内的字符串
    let quote_start = after.find('"')?;
    let rest = &after[quote_start + 1..];
    let quote_end = rest.find('"')?;
    Some(rest[..quote_end].to_string())
}

/// 提取 `类 名称` 的名称
fn extract_class(line: &str) -> Option<String> {
    let t = line.trim_start();
    if !t.starts_with("类") {
        return None;
    }
    let rest = t["类".len()..].trim_start();
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '{' || c == '(' || c == ':')
        .unwrap_or(rest.len());
    if end == 0 { return None; }
    Some(rest[..end].to_string())
}

/// 提取 `函 名称(` 的名称
fn extract_fn(line: &str) -> Option<String> {
    let t = line.trim_start();
    if !t.starts_with("函") {
        return None;
    }
    let rest = t["函".len()..].trim_start();
    let end = rest
        .find(|c: char| c == '(' || c.is_whitespace())
        .unwrap_or(rest.len());
    if end == 0 { return None; }
    Some(rest[..end].to_string())
}

/// 提取 `设 名称` 的名称
fn extract_let(line: &str) -> Option<String> {
    let t = line.trim_start();
    if !t.starts_with("设") {
        return None;
    }
    let rest = t["设".len()..].trim_start();
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '=')
        .unwrap_or(rest.len());
    if end == 0 { return None; }
    Some(rest[..end].to_string())
}

fn collect_lint_files() -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if std::path::Path::new("主函数.xt").exists() {
        files.push(PathBuf::from("主函数.xt"));
    }
    if std::path::Path::new("src").is_dir() {
        for e in std::fs::read_dir("src")? {
            let p = e?.path();
            if p.extension().map_or(false, |x| x == "xt") {
                files.push(p);
            }
        }
    }
    files.sort();
    Ok(files)
}
