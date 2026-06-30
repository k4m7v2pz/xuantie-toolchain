// build.rs for xuantie-cli
// 编译期把底层玄铁编译器 (Go 写的 main.go) 编出来,路径注入 env,运行时免去查找开销
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // cli/ 在 <root>/cli/,玄铁编译器在 <root>/
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root_dir = manifest_dir.join("..");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bin_dir = out_dir.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("创建 bin 目录失败");

    let exe_name = if cfg!(target_os = "windows") {
        "xuantie_core.exe"
    } else {
        "xuantie_core"
    };
    let exe_path = bin_dir.join(exe_name);

    // 找 go
    let go = which("go");
    match go {
        None => {
            println!("cargo:warning=未找到 go,底层玄铁编译器未编译");
            println!("cargo:warning=CLI 能运行但无法执行 .xt 文件");
            println!("cargo:rustc-env=XUANTIE_CORE_PATH=");
        }
        Some(go) => {
            // 静态链接,避免跨机器运行缺 dylib (尤其 macOS)
            let mut cmd = Command::new(&go);
            cmd.current_dir(&root_dir)
                .arg("build")
                .arg("-ldflags")
                .arg("-s -w")
                .arg("-o")
                .arg(&exe_path)
                .arg(".");

            // CGO 禁用,纯静态 (玄铁 build 子命令用 go build 转译路径,不依赖 cgo)
            cmd.env("CGO_ENABLED", "0");

            let status = cmd.status().unwrap_or_else(|e| {
                panic!("启动 go 编译器失败 {}: {}", go.display(), e);
            });
            if !status.success() {
                panic!("底层玄铁编译器 Go 编译失败");
            }
            println!("cargo:rustc-env=XUANTIE_CORE_PATH={}", exe_path.display());
            println!("cargo:rerun-if-changed={}", root_dir.join("main.go").display());
            println!("cargo:rerun-if-changed={}", root_dir.join("go.mod").display());
            // 主要源码目录,变了就重编
            for d in ["lexer", "parser", "ast", "compiler", "evaluator", "object", "token", "stdlib", "runtime"] {
                println!("cargo:rerun-if-changed={}", root_dir.join(d).display());
            }
        }
    }
}

fn which(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
        if cfg!(target_os = "windows") {
            let candidate = dir.join(format!("{}.exe", name));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}
