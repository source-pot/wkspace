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
