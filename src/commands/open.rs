use crate::context;
use crate::error::WkspaceError;
use std::env;
use std::process::Command;

pub fn run(name: &str) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let ctx = context::resolve(&cwd)?;
    let worktree_path = ctx.worktree_path(name);

    if !worktree_path.exists() {
        anyhow::bail!(WkspaceError::WorktreeNotFound(name.to_string()));
    }

    // Spawn subshell (skip in tests via env var)
    if env::var("WKSPACE_NO_SHELL").is_err() {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        println!("Opening shell in {}...", worktree_path.display());
        let mut child = Command::new(&shell)
            .current_dir(&worktree_path)
            .spawn()?;
        child.wait()?;
    }

    Ok(())
}
