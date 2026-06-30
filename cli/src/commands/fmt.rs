// xuantie fmt
// 玄铁目前无官方格式化器,此命令预留占位
// 上游玄铁语言在 1.0 自举定型前语法可能变动,格式化器推迟到那时再做
use anyhow::{anyhow, Result};

pub fn fmt(args: &[String]) -> Result<()> {
    let check_only = args.iter().any(|a| a == "--check" || a == "-c");
    if check_only {
        println!("✓ 玄铁暂无格式化器,--check 跳过 (格式化器将随 v1.0 自举推出)");
        return Ok(());
    }
    Err(anyhow!(
        "玄铁格式化器尚未实现\n\
         上游玄铁在 v1.0 自举定型前语法可能变动,格式化器推迟到那时再开发。\n\
         如需检查语法可用 `xuantie check`,检查结构可用 `xuantie lint`。"
    ))
}
