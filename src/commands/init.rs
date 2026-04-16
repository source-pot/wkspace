use crate::config::Config;
use crate::git;
use crate::hooks;
use std::collections::HashMap;
use std::env;
use std::path::Path;

pub fn run() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let repo_root = git::find_repo_root(&cwd)?;
    create_config(&repo_root)?;
    ensure_gitignore(&repo_root, ".worktrees")?;

    // Run user hook
    hooks::run_hook("post-init", &repo_root, &HashMap::new(), None);

    Ok(())
}

/// Create .wkspace.toml at the repo root if it doesn't exist.
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

/// Ensure the worktrees directory is in .gitignore.
pub fn ensure_gitignore(repo_root: &Path, entry: &str) -> anyhow::Result<()> {
    let gitignore_path = repo_root.join(".gitignore");
    let contents = if gitignore_path.exists() {
        std::fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    if contents.lines().any(|line| line.trim() == entry) {
        return Ok(());
    }

    let new_contents = if contents.is_empty() || contents.ends_with('\n') {
        format!("{contents}{entry}\n")
    } else {
        format!("{contents}\n{entry}\n")
    };
    std::fs::write(&gitignore_path, new_contents)?;
    Ok(())
}
