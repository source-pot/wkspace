use crate::context::Context;
use crate::git;
use crate::tui::tmux;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct WorktreeRow {
    pub name: String,
    pub branch: String,
    pub uncommitted: usize,
    pub last_commit: String,
    pub stale: bool,
    pub description: String,
    pub has_session: bool,
}

pub fn fetch_rows(ctx: &Context) -> anyhow::Result<Vec<WorktreeRow>> {
    let entries = git::list_worktrees(&ctx.repo_root)?;
    let worktrees_dir = ctx.worktrees_dir();
    let session = tmux::session_name(&ctx.repo_root);
    let active_windows = list_window_names(&session);
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let stale_threshold = (ctx.config.worktree.stale_days * 86400) as i64;

    let mut rows = Vec::new();
    for entry in entries
        .iter()
        .filter(|e| e.path.starts_with(&worktrees_dir))
    {
        let name = entry
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let branch = entry.branch.clone().unwrap_or_else(|| name.clone());

        let uncommitted = git::get_worktree_status(&entry.path)
            .map(|s| s.uncommitted_count)
            .unwrap_or(0);

        let (last_commit, stale) = match git::get_last_commit_time(&ctx.repo_root, &branch) {
            Some((rel, ts)) => (rel, (now_secs - ts) >= stale_threshold),
            None => ("-".to_string(), false),
        };

        let description = git::get_branch_description(&ctx.repo_root, &branch).unwrap_or_default();
        let has_session = active_windows.iter().any(|w| w == &name);

        rows.push(WorktreeRow {
            name,
            branch,
            uncommitted,
            last_commit,
            stale,
            description,
            has_session,
        });
    }
    Ok(rows)
}

fn list_window_names(session: &str) -> Vec<String> {
    let output = Command::new("tmux")
        .args(["list-windows", "-t", session, "-F", "#{window_name}"])
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|l| l.to_string())
            .collect(),
        _ => Vec::new(),
    }
}
