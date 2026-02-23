use crate::config::Config;
use crate::git;
use std::env;
use std::path::Path;

pub fn run() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let repo_root = git::find_repo_root(&cwd)?;
    create_config(&repo_root)
}

/// Create .wkspace.toml at the repo root if it doesn't exist.
/// Returns Ok(()) if file already exists (idempotent).
pub fn create_config(repo_root: &Path) -> anyhow::Result<()> {
    let config_path = repo_root.join(".wkspace.toml");
    if config_path.exists() {
        println!(".wkspace.toml already exists");
        return Ok(());
    }

    std::fs::write(&config_path, Config::default_template())?;
    println!("Created .wkspace.toml with defaults");
    Ok(())
}
