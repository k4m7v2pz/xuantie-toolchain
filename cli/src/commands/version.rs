// xuantie version / -V / --version
pub fn version(_args: &[String]) -> anyhow::Result<()> {
    println!("xuantie {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
