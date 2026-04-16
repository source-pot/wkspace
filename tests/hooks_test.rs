use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;
use wkspace::hooks;

#[test]
fn run_hook_executes_executable_script() {
    let hooks_dir = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    let marker = cwd.path().join("hook-ran");

    let hook_path = hooks_dir.path().join("post-new");
    std::fs::write(&hook_path, format!("#!/bin/sh\ntouch {}", marker.display())).unwrap();
    std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755)).unwrap();

    hooks::run_hook(
        "post-new",
        cwd.path(),
        &HashMap::new(),
        Some(hooks_dir.path()),
    );

    assert!(
        marker.exists(),
        "Hook script should have created marker file"
    );
}

#[test]
fn run_hook_passes_env_vars() {
    let hooks_dir = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    let out_file = cwd.path().join("env-out");

    let hook_path = hooks_dir.path().join("post-new");
    std::fs::write(
        &hook_path,
        format!("#!/bin/sh\necho $WORKTREE_NAME > {}", out_file.display()),
    )
    .unwrap();
    std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755)).unwrap();

    let mut env = HashMap::new();
    env.insert("WORKTREE_NAME".to_string(), "my-feature".to_string());

    hooks::run_hook("post-new", cwd.path(), &env, Some(hooks_dir.path()));

    let contents = std::fs::read_to_string(&out_file).unwrap();
    assert_eq!(contents.trim(), "my-feature");
}

#[test]
fn run_hook_skips_missing_hook_silently() {
    let hooks_dir = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();

    hooks::run_hook(
        "post-new",
        cwd.path(),
        &HashMap::new(),
        Some(hooks_dir.path()),
    );
}

#[test]
fn run_hook_skips_non_executable_file() {
    let hooks_dir = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    let marker = cwd.path().join("hook-ran");

    let hook_path = hooks_dir.path().join("post-new");
    std::fs::write(&hook_path, format!("#!/bin/sh\ntouch {}", marker.display())).unwrap();
    std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o644)).unwrap();

    hooks::run_hook(
        "post-new",
        cwd.path(),
        &HashMap::new(),
        Some(hooks_dir.path()),
    );

    assert!(!marker.exists(), "Non-executable hook should not have run");
}

#[test]
fn run_hook_warns_on_failure_does_not_panic() {
    let hooks_dir = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();

    let hook_path = hooks_dir.path().join("post-new");
    std::fs::write(&hook_path, "#!/bin/sh\nexit 1").unwrap();
    std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755)).unwrap();

    hooks::run_hook(
        "post-new",
        cwd.path(),
        &HashMap::new(),
        Some(hooks_dir.path()),
    );
}

#[test]
fn run_hook_skips_when_hooks_dir_missing() {
    let cwd = TempDir::new().unwrap();
    let nonexistent = std::path::Path::new("/tmp/wkspace-nonexistent-hooks-dir");

    hooks::run_hook("post-new", cwd.path(), &HashMap::new(), Some(nonexistent));
}

#[test]
fn run_hook_runs_in_specified_cwd() {
    let hooks_dir = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    let out_file = cwd.path().join("pwd-out");

    let hook_path = hooks_dir.path().join("post-new");
    std::fs::write(
        &hook_path,
        format!("#!/bin/sh\npwd > {}", out_file.display()),
    )
    .unwrap();
    std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755)).unwrap();

    hooks::run_hook(
        "post-new",
        cwd.path(),
        &HashMap::new(),
        Some(hooks_dir.path()),
    );

    let contents = std::fs::read_to_string(&out_file).unwrap();
    let expected = cwd.path().canonicalize().unwrap();
    let actual = std::path::PathBuf::from(contents.trim())
        .canonicalize()
        .unwrap();
    assert_eq!(actual, expected);
}
