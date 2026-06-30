// xuantie new <名称> / xuantie init
// 脚手架:创建项目结构 + 入口文件 + xuantie.toml
use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

const XUANTIE_TOML: &str = r#"# 玄铁项目配置
[package]
name = "{name}"
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
"#;

const MAIN_XT: &str = "// 主函数.xt — 玄铁项目入口\n\n示(\"你好，玄铁！\")\n";

const GITIGNORE: &str = r#"dist/
*.exe
*.ll
*.o
tiepm_modules/
xuantie.lock
"#;

/// xuantie new <名称> — 在当前目录下创建子目录
pub fn create(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("用法: xuantie new <项目名称>"));
    }
    let name = &args[0];
    let target = Path::new(name);
    if target.exists() {
        return Err(anyhow!("目录已存在: {}", name));
    }

    fs::create_dir_all(target.join("src"))?;
    fs::create_dir_all(target.join("tests"))?;
    fs::write(target.join("主函数.xt"), MAIN_XT)?;
    fs::write(
        target.join("xuantie.toml"),
        XUANTIE_TOML.replace("{name}", name),
    )?;
    fs::write(target.join(".gitignore"), GITIGNORE)?;

    println!("✓ 已创建项目: {}", name);
    println!("  cd {} && xuantie run", name);
    Ok(())
}

/// xuantie init — 在当前空目录初始化项目
pub fn init(_args: &[String]) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let name = cwd
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("xuantie-project")
        .to_string();

    if Path::new("xuantie.toml").exists() {
        return Err(anyhow!("当前目录已是玄铁项目 (xuantie.toml 已存在)"));
    }

    fs::create_dir_all("src")?;
    fs::create_dir_all("tests")?;
    if !Path::new("主函数.xt").exists() {
        fs::write("主函数.xt", MAIN_XT)?;
    }
    fs::write("xuantie.toml", XUANTIE_TOML.replace("{name}", &name))?;
    fs::write(".gitignore", GITIGNORE)?;

    println!("✓ 已在当前目录初始化玄铁项目: {}", name);
    println!("  xuantie run");
    Ok(())
}
