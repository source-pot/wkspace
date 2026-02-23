use crate::error::WkspaceError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Find the root of the git repository containing `start_dir`.
pub fn find_repo_root(start_dir: &Path) -> anyhow::Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(start_dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(WkspaceError::NotAGitRepo);
    }

    let path = String::from_utf8(output.stdout)?
        .trim()
        .to_string();
    Ok(PathBuf::from(path))
}

/// Check if a local branch exists.
pub fn branch_exists(repo_root: &Path, branch: &str) -> anyhow::Result<bool> {
    let output = Command::new("git")
        .args(["branch", "--list", branch])
        .current_dir(repo_root)
        .output()?;
    Ok(!String::from_utf8(output.stdout)?.trim().is_empty())
}

/// Create a new worktree with a new branch based on `base_branch`.
pub fn add_worktree(
    repo_root: &Path,
    worktree_path: &Path,
    branch: &str,
    base_branch: &str,
) -> anyhow::Result<()> {
    if branch_exists(repo_root, branch)? {
        anyhow::bail!(WkspaceError::BranchExists(branch.to_string()));
    }

    // Ensure parent directory exists
    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            &worktree_path.to_string_lossy(),
            "-b",
            branch,
            base_branch,
        ])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(WkspaceError::GitError(stderr.trim().to_string()));
    }
    Ok(())
}

/// Force-delete a branch.
pub fn delete_branch(repo_root: &Path, branch: &str) -> anyhow::Result<()> {
    let output = Command::new("git")
        .args(["branch", "-D", branch])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(WkspaceError::GitError(stderr.trim().to_string()));
    }
    Ok(())
}

/// Prune stale worktree references.
pub fn prune_worktrees(repo_root: &Path) -> anyhow::Result<()> {
    let output = Command::new("git")
        .args(["worktree", "prune"])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(WkspaceError::GitError(stderr.trim().to_string()));
    }
    Ok(())
}

/// A parsed worktree entry from `git worktree list --porcelain`.
#[derive(Debug)]
pub struct WorktreeEntry {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub bare: bool,
}

/// List all worktrees in the repository.
pub fn list_worktrees(repo_root: &Path) -> anyhow::Result<Vec<WorktreeEntry>> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(WkspaceError::GitError(stderr.trim().to_string()));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let mut entries = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch: Option<String> = None;
    let mut current_bare = false;

    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            // Save previous entry if any
            if let Some(p) = current_path.take() {
                entries.push(WorktreeEntry {
                    path: p,
                    branch: current_branch.take(),
                    bare: current_bare,
                });
                current_bare = false;
            }
            current_path = Some(PathBuf::from(path));
        } else if let Some(branch_ref) = line.strip_prefix("branch ") {
            // branch refs/heads/main -> main
            current_branch = branch_ref.strip_prefix("refs/heads/").map(|s| s.to_string());
        } else if line == "bare" {
            current_bare = true;
        }
    }

    // Don't forget the last entry
    if let Some(p) = current_path {
        entries.push(WorktreeEntry {
            path: p,
            branch: current_branch,
            bare: current_bare,
        });
    }

    Ok(entries)
}
