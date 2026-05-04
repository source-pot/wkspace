use std::path::Path;

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
