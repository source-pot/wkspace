use std::path::Path;
use std::process::Command;

const REQUIRED_TMUX: (u32, u32) = (3, 0);

#[derive(Debug, thiserror::Error)]
pub enum TmuxError {
    #[error("tmux is not installed or not in PATH")]
    NotFound,
    #[error("tmux version {0}.{1} is too old (need >= {}.{})", REQUIRED_TMUX.0, REQUIRED_TMUX.1)]
    TooOld(u32, u32),
    #[error("could not parse tmux version output: {0}")]
    UnparseableVersion(String),
}

pub fn repo_slug(repo_root: &Path) -> String {
    let raw = repo_root.file_name().and_then(|f| f.to_str()).unwrap_or("");

    let mut slug = String::with_capacity(raw.len());
    let mut last_was_dash = false;
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() {
            for lc in c.to_lowercase() {
                slug.push(lc);
            }
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "repo".to_string()
    } else {
        trimmed
    }
}

pub fn session_name(repo_root: &Path) -> String {
    format!("wkspace-{}", repo_slug(repo_root))
}

/// Parse output of `tmux -V`, e.g. "tmux 3.4" or "tmux next-3.5".
pub fn parse_version_output(s: &str) -> Option<(u32, u32)> {
    let trimmed = s.trim();
    let after_prefix = trimmed.strip_prefix("tmux ")?;
    let stripped = after_prefix.strip_prefix("next-").unwrap_or(after_prefix);
    let mut parts = stripped.split('.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor_part = parts.next()?;
    // Strip any trailing non-digits (e.g. "3a")
    let minor_str: String = minor_part
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let minor: u32 = minor_str.parse().ok()?;
    Some((major, minor))
}

pub fn version_at_least(actual: (u32, u32), required: (u32, u32)) -> bool {
    actual.0 > required.0 || (actual.0 == required.0 && actual.1 >= required.1)
}

pub fn preflight() -> Result<(u32, u32), TmuxError> {
    let output = Command::new("tmux")
        .arg("-V")
        .output()
        .map_err(|_| TmuxError::NotFound)?;

    let raw = String::from_utf8_lossy(&output.stdout).to_string();
    let version =
        parse_version_output(&raw).ok_or_else(|| TmuxError::UnparseableVersion(raw.clone()))?;

    if !version_at_least(version, REQUIRED_TMUX) {
        return Err(TmuxError::TooOld(version.0, version.1));
    }
    Ok(version)
}
