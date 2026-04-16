use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Return the hooks directory.
/// Checks `WKSPACE_HOOKS_DIR` env var first (for testing), then `~/.config/wkspace/hooks`.
fn default_hooks_dir() -> Option<std::path::PathBuf> {
    if let Ok(dir) = std::env::var("WKSPACE_HOOKS_DIR") {
        return Some(std::path::PathBuf::from(dir));
    }
    std::env::var("HOME").ok().map(|home| {
        std::path::PathBuf::from(home)
            .join(".config")
            .join("wkspace")
            .join("hooks")
    })
}

/// Run a user hook if it exists and is executable.
/// Prints a warning to stderr on failure — never returns an error.
///
/// The `hooks_dir_override` parameter is for testing; pass `None` in production
/// to use the default `~/.config/wkspace/hooks` directory.
pub fn run_hook(
    hook_name: &str,
    cwd: &Path,
    extra_env: &HashMap<String, String>,
    hooks_dir_override: Option<&Path>,
) {
    let hooks_dir = match hooks_dir_override {
        Some(d) => d.to_path_buf(),
        None => match default_hooks_dir() {
            Some(d) => d,
            None => return,
        },
    };

    let hook_path = hooks_dir.join(hook_name);

    if !hook_path.exists() {
        return;
    }

    #[cfg(unix)]
    {
        let metadata = match std::fs::metadata(&hook_path) {
            Ok(m) => m,
            Err(_) => return,
        };
        if metadata.permissions().mode() & 0o111 == 0 {
            eprintln!(
                "Warning: hook '{}' exists but is not executable, skipping",
                hook_path.display()
            );
            return;
        }
    }

    println!("Running hook: {hook_name}");
    match Command::new(&hook_path)
        .current_dir(cwd)
        .envs(extra_env)
        .status()
    {
        Ok(status) if status.success() => {}
        Ok(status) => {
            eprintln!(
                "Warning: hook '{hook_name}' exited with code {}",
                status
                    .code()
                    .map_or("unknown".to_string(), |c| c.to_string())
            );
        }
        Err(e) => {
            eprintln!("Warning: failed to run hook '{hook_name}': {e}");
        }
    }
}
