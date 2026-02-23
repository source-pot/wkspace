use crate::context;
use crate::git;
use std::env;

pub fn run() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let ctx = context::resolve(&cwd)?;
    let worktrees_dir = ctx.worktrees_dir();

    let entries = git::list_worktrees(&ctx.repo_root)?;

    // Filter to only managed worktrees (those under the worktrees directory)
    let managed: Vec<_> = entries
        .iter()
        .filter(|e| e.path.starts_with(&worktrees_dir))
        .collect();

    if managed.is_empty() {
        println!("No worktrees");
        return Ok(());
    }

    for entry in &managed {
        let name = entry
            .path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        let branch = entry.branch.as_deref().unwrap_or("(detached)");
        println!("  {name}\t{branch}\t{}", entry.path.display());
    }

    Ok(())
}
