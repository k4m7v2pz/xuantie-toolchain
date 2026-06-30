// xuantie add <库名> <git url> [--tag/--branch/--rev/--path]
// 把依赖写进 xuantie.toml 并拉到 tiepm_modules/,同时更新 xuantie.lock
use anyhow::{anyhow, Result};
use std::process::Command;

use crate::lock::{record_dep, dep_local_path, Lock};
use crate::manifest::{Dependency, Manifest};

pub fn add(args: &[String]) -> Result<()> {
    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if positional.is_empty() {
        return Err(anyhow!(
            "用法:\n\
             \x20 xuantie add <库名> <git url> [--tag <t> | --branch <b> | --rev <r>]\n\
             \x20 xuantie add <库名> --path <本地路径>"
        ));
    }
    let alias = positional[0];

    let path_val = take_value(args, &["--path"]);
    let tag_val = take_value(args, &["--tag"]);
    let branch_val = take_value(args, &["--branch"]);
    let rev_val = take_value(args, &["--rev"]);

    if path_val.is_some() {
        let has_git_url = positional.len() > 1
            && positional.get(1).map(|s| s.as_str()) != path_val.as_deref();
        if has_git_url {
            return Err(anyhow!("--path 与 git url 不能同时指定"));
        }
    }
    let dep = if let Some(p) = path_val {
        Dependency::Path { path: p }
    } else {
        let git = positional
            .get(1)
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("缺少 git url (用法: xuantie add <库名> <git url> [--tag/--branch/--rev] 或 --path <本地路径>)"))?;
        let ref_count = [tag_val.is_some(), branch_val.is_some(), rev_val.is_some()]
            .iter()
            .filter(|x| **x)
            .count();
        if ref_count > 1 {
            return Err(anyhow!("--tag/--branch/--rev 只能选一个"));
        }
        Dependency::Git {
            git,
            tag: tag_val,
            branch: branch_val,
            rev: rev_val,
        }
    };

    let commit = materialize(alias, &dep)?;
    write_dep_to_manifest(alias, &dep)?;

    let mut lock = Lock::load_current()?;
    record_dep(&mut lock, alias, &dep, commit.clone());
    lock.save_current()?;

    println!("✓ 已添加依赖: {}", alias);
    println!("  来源: {}", dep.source_url());
    if let Some(c) = commit {
        println!("  锁定 commit: {}", c);
    }
    match &dep {
        Dependency::Path { path } => println!("  本地路径: {}", path),
        _ => println!("  本地路径: {}", dep_local_path(alias).display()),
    }
    Ok(())
}

/// 把依赖拉到 tiepm_modules/<别名>/
fn materialize(alias: &str, dep: &Dependency) -> Result<Option<String>> {
    let local = dep_local_path(alias);

    match dep {
        Dependency::Path { .. } => {
            if let Dependency::Path { path } = dep {
                if !std::path::Path::new(path).exists() {
                    return Err(anyhow!("本地依赖路径不存在: {}", path));
                }
            }
            Ok(None)
        }
        Dependency::Git { git, tag, branch, rev } => {
            if local.exists() {
                std::fs::remove_dir_all(&local)
                    .map_err(|e| anyhow!("清理旧依赖目录失败 {}: {}", local.display(), e))?;
            }
            std::fs::create_dir_all("tiepm_modules")
                .map_err(|e| anyhow!("创建 tiepm_modules/ 失败: {}", e))?;

            let mut cmd = Command::new("git");
            cmd.arg("clone").arg("--depth").arg("50").arg(git).arg(&local);
            if let Some(t) = tag {
                cmd.arg("--branch").arg(t);
            } else if let Some(b) = branch {
                cmd.arg("--branch").arg(b);
            }
            run_git(&mut cmd)?;

            if let Some(r) = rev {
                run_git(Command::new("git").arg("fetch").arg("--depth").arg("1").arg(git).arg(r).current_dir(&local))?;
                run_git(Command::new("git").arg("checkout").arg(r).current_dir(&local))?;
            }

            let out = Command::new("git")
                .arg("rev-parse").arg("HEAD")
                .current_dir(&local)
                .output()
                .map_err(|e| anyhow!("git rev-parse 失败: {}", e))?;
            if !out.status.success() {
                return Err(anyhow!("获取 commit hash 失败"));
            }
            let hash = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let _ = std::fs::remove_dir_all(local.join(".git"));
            Ok(Some(hash))
        }
        Dependency::Version(v) => Err(anyhow!(
            "纯版本号依赖 '{}' 暂不支持 (无包注册表);请用 git 或 path 形式",
            v
        )),
    }
}

fn run_git(cmd: &mut Command) -> Result<()> {
    let out = cmd.output().map_err(|e| anyhow!("启动 git 失败: {}", e))?;
    if !out.status.success() {
        return Err(anyhow!(
            "git 操作失败 (exit {}): {}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(())
}

fn take_value(args: &[String], flags: &[&str]) -> Option<String> {
    for (i, a) in args.iter().enumerate() {
        for f in flags {
            if a == *f {
                return args.get(i + 1).cloned();
            }
        }
    }
    None
}

/// 把依赖追加到 xuantie.toml 的 [dependencies] 段
fn write_dep_to_manifest(alias: &str, dep: &Dependency) -> Result<()> {
    let path = std::path::Path::new("xuantie.toml");
    if !path.exists() {
        return Err(anyhow!("未找到 xuantie.toml,请先在项目目录运行"));
    }
    let original = std::fs::read_to_string(path)?;
    let dep_line = format_dep_line(alias, dep);

    let bare = format!("{} =", alias);
    let quoted = format!("\"{}\" =", alias);
    let bare2 = format!("{}=", alias);
    let quoted2 = format!("\"{}\"=", alias);
    let cleaned: Vec<&str> = original
        .lines()
        .filter(|line| {
            let t = line.trim_start();
            !t.starts_with(&bare) && !t.starts_with(&bare2)
                && !t.starts_with(&quoted) && !t.starts_with(&quoted2)
        })
        .collect();

    let dep_section_idx = cleaned
        .iter()
        .position(|l| l.trim() == "[dependencies]");

    let mut new_lines: Vec<String> = cleaned.iter().map(|s| s.to_string()).collect();
    match dep_section_idx {
        Some(idx) => {
            let mut insert_at = idx + 1;
            while insert_at < new_lines.len() && !new_lines[insert_at].trim_start().starts_with('[') {
                insert_at += 1;
            }
            new_lines.insert(insert_at, dep_line);
        }
        None => {
            if !new_lines.is_empty() && !new_lines.last().map_or(false, |s| s.is_empty()) {
                new_lines.push(String::new());
            }
            new_lines.push("[dependencies]".into());
            new_lines.push(dep_line);
        }
    }

    let trailing_nl = original.ends_with('\n');
    let mut out = new_lines.join("\n");
    if trailing_nl {
        out.push('\n');
    }
    std::fs::write(path, out)?;
    Ok(())
}

fn format_dep_line(alias: &str, dep: &Dependency) -> String {
    let needs_quote = alias.chars().any(|c| !c.is_ascii());
    let key = if needs_quote {
        format!("\"{}\"", alias)
    } else {
        alias.to_string()
    };
    match dep {
        Dependency::Git { git, tag, branch, rev } => {
            let mut parts = vec![format!("git = \"{}\"", git)];
            if let Some(t) = tag {
                parts.push(format!("tag = \"{}\"", t));
            }
            if let Some(b) = branch {
                parts.push(format!("branch = \"{}\"", b));
            }
            if let Some(r) = rev {
                parts.push(format!("rev = \"{}\"", r));
            }
            format!("{} = {{ {} }}", key, parts.join(", "))
        }
        Dependency::Path { path } => format!("{} = {{ path = \"{}\" }}", key, path),
        Dependency::Version(_) => unreachable!("materialize 已拦掉 Version"),
    }
}

/// 供 run/build 复用:确保所有 manifest 依赖都已拉到本地且与 lock 一致
pub fn ensure_deps_ready(manifest: &Manifest) -> (bool, Vec<String>) {
    let lock = Lock::load_current().unwrap_or_default();
    let (consistent, drift) = lock.consistency_with(manifest);
    if !consistent {
        let msgs = drift
            .iter()
            .map(|a| format!("{} (未拉取或与 lock 不一致,请跑 `xuantie add {} <url>` 或重新 add)", a, a))
            .collect();
        return (false, msgs);
    }
    let mut missing = Vec::new();
    for (alias, dep) in &manifest.dependencies {
        if dep.is_path() {
            continue;
        }
        if !dep_local_path(alias).exists() {
            missing.push(format!("{} (tiepm_modules/{} 缺失,请跑 `xuantie add {} <url>`)", alias, alias, alias));
        }
    }
    (missing.is_empty(), missing)
}
