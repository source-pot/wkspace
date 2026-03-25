use crate::commands::new;
use crate::context;
use crate::error::WkspaceError;
use std::collections::HashMap;
use std::env;

pub fn run(name: &str) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let ctx = context::resolve(&cwd)?;
    let worktree_path = ctx.worktree_path(name);

    if !worktree_path.exists() {
        anyhow::bail!(WkspaceError::WorktreeNotFound(name.to_string()));
    }

    // Spawn subshell (skip in tests via env var)
    if env::var("WKSPACE_NO_SHELL").is_err() {
        new::spawn_shell(&worktree_path, &HashMap::new())?;
    }

    Ok(())
}
