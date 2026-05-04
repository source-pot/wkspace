# TUI Milestone 1 — Skeleton (read-only) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a tmux-backed read-only TUI: `wkspace` (no args) launches a ratatui sidebar inside a repo-scoped tmux session, showing worktree list + detail + keymap. Navigation, refresh, help, detach. No mutating actions yet.

**Architecture:** Two binaries in one: the existing `wkspace` CLI gets a new no-args path (the launcher) that creates/attaches to a tmux session named `wkspace-<repo-slug>`, and a hidden `wkspace __controller` subcommand that runs the ratatui app inside the controller pane. tmux is the multiplexer. The controller derives all state from `git` and `tmux` subprocess calls — no separate state store.

**Tech Stack:** Rust 2021, ratatui 0.29, crossterm 0.28, anyhow, existing wkspace modules (`git`, `context`, `config`).

**Spec reference:** `docs/superpowers/specs/2026-05-04-tui-redesign-design.md` (M1 section).

---

## File structure

**New files:**
- `src/tui/mod.rs` — module exports + `pub fn run_launcher()`.
- `src/tui/tmux.rs` — pure helpers: slug, in-tmux detection, command builders.
- `src/tui/launcher.rs` — orchestrates session detect → action (attach / create / refocus / error).
- `src/tui/controller/mod.rs` — App state + main loop.
- `src/tui/controller/data.rs` — derive `Vec<WorktreeRow>` from git/tmux.
- `src/tui/controller/view.rs` — pure rendering function (`fn render(state, frame)`).
- `src/tui/controller/events.rs` — pure key-to-action mapping.
- `tests/tui_tmux_test.rs` — slug + command builder unit tests.
- `tests/tui_events_test.rs` — event-handler dispatch tests.
- `tests/tui_view_test.rs` — TestBackend snapshots of layout.
- `tests/tui_launcher_test.rs` — launcher decision logic (no real tmux).

**Modified files:**
- `Cargo.toml` — add `ratatui`, `crossterm`.
- `src/lib.rs` — add `pub mod tui;`.
- `src/main.rs` — add no-args launcher path + hidden `__controller` subcommand.

---

## Conventions

- All TDD: failing test → minimal impl → passing test → commit.
- Commit after each task ends with all tests/clippy/fmt clean.
- Use `cargo test --locked`, `cargo clippy --locked -- -D warnings`, `cargo fmt --check` per CLAUDE.md.
- For each test that depends on tmux being present, gate with `#[cfg_attr(not(feature = "tmux-tests"), ignore)]` or guard with `which::which("tmux").is_ok()` and skip; we won't add tmux as a CI dependency for M1.

---

## Task 1: Add ratatui + crossterm dependencies

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add deps**

In `Cargo.toml`, under `[dependencies]`, add:

```toml
ratatui = "0.29"
crossterm = "0.28"
```

- [ ] **Step 2: Verify build**

Run: `cargo build --locked`
Expected: builds successfully, no warnings.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "build: add ratatui and crossterm for TUI skeleton"
```

---

## Task 2: Create empty tui module skeleton

**Files:**
- Create: `src/tui/mod.rs`
- Create: `src/tui/tmux.rs`
- Create: `src/tui/launcher.rs`
- Create: `src/tui/controller/mod.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Create empty module files**

`src/tui/mod.rs`:

```rust
pub mod controller;
pub mod launcher;
pub mod tmux;
```

`src/tui/tmux.rs`:

```rust
// tmux helper functions live here.
```

`src/tui/launcher.rs`:

```rust
// Launcher decision logic + orchestration lives here.
```

`src/tui/controller/mod.rs`:

```rust
// Controller app state + main loop lives here.
```

- [ ] **Step 2: Wire `tui` into `src/lib.rs`**

`src/lib.rs` becomes:

```rust
pub mod commands;
pub mod config;
pub mod context;
pub mod error;
pub mod git;
pub mod hooks;
pub mod ports;
pub mod scripts;
pub mod tui;
```

- [ ] **Step 3: Verify build**

Run: `cargo build --locked && cargo clippy --locked -- -D warnings`
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add src/lib.rs src/tui/
git commit -m "feat(tui): add empty tui module skeleton"
```

---

## Task 3: Implement repo slug logic (TDD)

The slug is used for the tmux session name: `wkspace-<slug>`. It's derived from the absolute path's final component, lowercased, with non-alphanumerics → `-`, collapsed dashes, no leading/trailing dash.

**Files:**
- Create: `tests/tui_tmux_test.rs`
- Modify: `src/tui/tmux.rs`

- [ ] **Step 1: Write failing test**

`tests/tui_tmux_test.rs`:

```rust
use std::path::Path;
use wkspace::tui::tmux;

#[test]
fn slug_lowercases_and_replaces_non_alnum() {
    assert_eq!(tmux::repo_slug(Path::new("/Users/a/My Repo")), "my-repo");
}

#[test]
fn slug_collapses_repeated_dashes() {
    assert_eq!(tmux::repo_slug(Path::new("/x/foo___bar.git")), "foo-bar-git");
}

#[test]
fn slug_trims_leading_and_trailing_dashes() {
    assert_eq!(tmux::repo_slug(Path::new("/x/--weird--")), "weird");
}

#[test]
fn slug_falls_back_when_empty() {
    assert_eq!(tmux::repo_slug(Path::new("/")), "repo");
}
```

- [ ] **Step 2: Run test, verify failure**

Run: `cargo test --test tui_tmux_test --locked`
Expected: fails (`repo_slug` not found).

- [ ] **Step 3: Implement**

In `src/tui/tmux.rs`:

```rust
use std::path::Path;

pub fn repo_slug(repo_root: &Path) -> String {
    let raw = repo_root
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("");

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
```

- [ ] **Step 4: Run test, verify pass**

Run: `cargo test --test tui_tmux_test --locked`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add src/tui/tmux.rs tests/tui_tmux_test.rs
git commit -m "feat(tui): add repo slug + session name helpers"
```

---

## Task 4: Tmux preflight check

Detect tmux binary + version ≥ 3.0. Returns `Result<(u32, u32), TmuxError>`.

**Files:**
- Modify: `src/tui/tmux.rs`
- Modify: `tests/tui_tmux_test.rs`

- [ ] **Step 1: Write failing tests for version parsing**

Append to `tests/tui_tmux_test.rs`:

```rust
#[test]
fn parses_modern_version() {
    assert_eq!(tmux::parse_version_output("tmux 3.4\n"), Some((3, 4)));
}

#[test]
fn parses_legacy_version() {
    assert_eq!(tmux::parse_version_output("tmux 2.8\n"), Some((2, 8)));
}

#[test]
fn parses_next_version() {
    assert_eq!(tmux::parse_version_output("tmux next-3.5\n"), Some((3, 5)));
}

#[test]
fn ignores_unparseable() {
    assert_eq!(tmux::parse_version_output("garbage"), None);
}

#[test]
fn version_at_least_compares_correctly() {
    assert!(tmux::version_at_least((3, 0), (3, 0)));
    assert!(tmux::version_at_least((3, 4), (3, 0)));
    assert!(tmux::version_at_least((4, 0), (3, 9)));
    assert!(!tmux::version_at_least((2, 9), (3, 0)));
}
```

- [ ] **Step 2: Run, verify fail**

Run: `cargo test --test tui_tmux_test --locked`
Expected: 5 new test failures (functions don't exist).

- [ ] **Step 3: Implement parsers**

Append to `src/tui/tmux.rs`:

```rust
/// Parse output of `tmux -V`, e.g. "tmux 3.4" or "tmux next-3.5".
pub fn parse_version_output(s: &str) -> Option<(u32, u32)> {
    let trimmed = s.trim();
    let after_prefix = trimmed.strip_prefix("tmux ")?;
    let stripped = after_prefix.strip_prefix("next-").unwrap_or(after_prefix);
    let mut parts = stripped.split('.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor_part = parts.next()?;
    // Strip any trailing non-digits (e.g. "3a")
    let minor_str: String = minor_part.chars().take_while(|c| c.is_ascii_digit()).collect();
    let minor: u32 = minor_str.parse().ok()?;
    Some((major, minor))
}

pub fn version_at_least(actual: (u32, u32), required: (u32, u32)) -> bool {
    actual.0 > required.0 || (actual.0 == required.0 && actual.1 >= required.1)
}
```

- [ ] **Step 4: Run, verify pass**

Run: `cargo test --test tui_tmux_test --locked`
Expected: all pass.

- [ ] **Step 5: Add the actual preflight function (not unit-tested — it shells out)**

Append to `src/tui/tmux.rs`:

```rust
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

pub fn preflight() -> Result<(u32, u32), TmuxError> {
    let output = Command::new("tmux")
        .arg("-V")
        .output()
        .map_err(|_| TmuxError::NotFound)?;

    let raw = String::from_utf8_lossy(&output.stdout).to_string();
    let version = parse_version_output(&raw).ok_or_else(|| TmuxError::UnparseableVersion(raw.clone()))?;

    if !version_at_least(version, REQUIRED_TMUX) {
        return Err(TmuxError::TooOld(version.0, version.1));
    }
    Ok(version)
}
```

- [ ] **Step 6: Add `thiserror` dep if not present**

Check `Cargo.toml`. If `thiserror` is missing, add:

```toml
thiserror = "1"
```

(Run `cargo build` and confirm no errors.)

- [ ] **Step 7: Run full check**

Run: `cargo test --locked && cargo clippy --locked -- -D warnings && cargo fmt --check`
Expected: clean.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock src/tui/tmux.rs tests/tui_tmux_test.rs
git commit -m "feat(tui): add tmux preflight + version detection"
```

---

## Task 5: Tmux state detection

Pure function `tmux_env_state(env_lookup) -> TmuxEnv`. Extract `TMUX` env var presence; current session via `tmux display-message`.

**Files:**
- Modify: `src/tui/tmux.rs`
- Modify: `tests/tui_tmux_test.rs`

- [ ] **Step 1: Write tests**

Append to `tests/tui_tmux_test.rs`:

```rust
#[test]
fn outside_tmux_when_env_missing() {
    let lookup = |_: &str| None;
    assert!(matches!(tmux::detect_env(&lookup), tmux::TmuxEnv::Outside));
}

#[test]
fn inside_tmux_when_env_set() {
    let lookup = |k: &str| if k == "TMUX" { Some("/tmp/tmux-1000/default,1234,0".to_string()) } else { None };
    assert!(matches!(tmux::detect_env(&lookup), tmux::TmuxEnv::Inside));
}
```

- [ ] **Step 2: Run, verify fail**

- [ ] **Step 3: Implement**

Append to `src/tui/tmux.rs`:

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum TmuxEnv {
    Inside,
    Outside,
}

pub fn detect_env<F: Fn(&str) -> Option<String>>(lookup: &F) -> TmuxEnv {
    match lookup("TMUX") {
        Some(s) if !s.is_empty() => TmuxEnv::Inside,
        _ => TmuxEnv::Outside,
    }
}

/// Returns the current tmux session name, when run from inside a pane.
pub fn current_session_name() -> Option<String> {
    let output = Command::new("tmux")
        .args(["display-message", "-p", "#{session_name}"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Returns true if a tmux session with `name` exists.
pub fn session_exists(name: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
```

- [ ] **Step 4: Run, verify pass**

Run: `cargo test --test tui_tmux_test --locked`

- [ ] **Step 5: Commit**

```bash
git add src/tui/tmux.rs tests/tui_tmux_test.rs
git commit -m "feat(tui): add tmux env detection helpers"
```

---

## Task 6: Launcher decision logic (pure)

Computes the next action from inputs. No side effects.

**Files:**
- Modify: `src/tui/launcher.rs`
- Create: `tests/tui_launcher_test.rs`

- [ ] **Step 1: Write tests**

`tests/tui_launcher_test.rs`:

```rust
use wkspace::tui::launcher::{decide, Decision, Inputs};
use wkspace::tui::tmux::TmuxEnv;

#[test]
fn outside_tmux_no_session_creates() {
    let d = decide(Inputs {
        env: TmuxEnv::Outside,
        target_session: "wkspace-foo".into(),
        current_session: None,
        target_session_exists: false,
    });
    assert_eq!(d, Decision::CreateAndAttach);
}

#[test]
fn outside_tmux_session_exists_attaches() {
    let d = decide(Inputs {
        env: TmuxEnv::Outside,
        target_session: "wkspace-foo".into(),
        current_session: None,
        target_session_exists: true,
    });
    assert_eq!(d, Decision::Attach);
}

#[test]
fn inside_target_session_refocuses() {
    let d = decide(Inputs {
        env: TmuxEnv::Inside,
        target_session: "wkspace-foo".into(),
        current_session: Some("wkspace-foo".into()),
        target_session_exists: true,
    });
    assert_eq!(d, Decision::RefocusController);
}

#[test]
fn inside_other_session_errors() {
    let d = decide(Inputs {
        env: TmuxEnv::Inside,
        target_session: "wkspace-foo".into(),
        current_session: Some("mywork".into()),
        target_session_exists: false,
    });
    assert!(matches!(d, Decision::ErrorInsideOtherSession { target_exists: false }));
}

#[test]
fn inside_other_session_target_exists_errors_with_hint() {
    let d = decide(Inputs {
        env: TmuxEnv::Inside,
        target_session: "wkspace-foo".into(),
        current_session: Some("mywork".into()),
        target_session_exists: true,
    });
    assert!(matches!(d, Decision::ErrorInsideOtherSession { target_exists: true }));
}
```

- [ ] **Step 2: Run, verify fail**

- [ ] **Step 3: Implement**

`src/tui/launcher.rs`:

```rust
use crate::tui::tmux::TmuxEnv;

pub struct Inputs {
    pub env: TmuxEnv,
    pub target_session: String,
    pub current_session: Option<String>,
    pub target_session_exists: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    CreateAndAttach,
    Attach,
    RefocusController,
    ErrorInsideOtherSession { target_exists: bool },
}

pub fn decide(inputs: Inputs) -> Decision {
    match inputs.env {
        TmuxEnv::Outside => {
            if inputs.target_session_exists {
                Decision::Attach
            } else {
                Decision::CreateAndAttach
            }
        }
        TmuxEnv::Inside => {
            if inputs.current_session.as_deref() == Some(inputs.target_session.as_str()) {
                Decision::RefocusController
            } else {
                Decision::ErrorInsideOtherSession {
                    target_exists: inputs.target_session_exists,
                }
            }
        }
    }
}
```

- [ ] **Step 4: Run, verify pass**

- [ ] **Step 5: Commit**

```bash
git add src/tui/launcher.rs tests/tui_launcher_test.rs
git commit -m "feat(tui): add launcher decision function"
```

---

## Task 7: Launcher orchestration (side-effecting wrapper)

Reads env, queries tmux, calls `decide()`, executes the action.

**Files:**
- Modify: `src/tui/launcher.rs`
- Modify: `src/tui/mod.rs`

- [ ] **Step 1: Add the orchestrator**

Append to `src/tui/launcher.rs`:

```rust
use crate::tui::tmux;
use anyhow::Context;
use std::path::Path;
use std::process::Command;

pub fn run(repo_root: &Path) -> anyhow::Result<()> {
    tmux::preflight().context("tmux preflight failed")?;

    let target = tmux::session_name(repo_root);
    let env = tmux::detect_env(&|k| std::env::var(k).ok());
    let current = if env == tmux::TmuxEnv::Inside { tmux::current_session_name() } else { None };
    let exists = tmux::session_exists(&target);

    let decision = decide(Inputs {
        env: env,
        target_session: target.clone(),
        current_session: current,
        target_session_exists: exists,
    });

    match decision {
        Decision::CreateAndAttach => create_and_attach(&target, repo_root),
        Decision::Attach => attach(&target),
        Decision::RefocusController => refocus_controller(&target),
        Decision::ErrorInsideOtherSession { target_exists } => {
            print_inside_other_session_error(&target, target_exists);
            std::process::exit(1);
        }
    }
}

fn current_exe_str() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "wkspace".to_string())
}

fn create_and_attach(target: &str, repo_root: &Path) -> anyhow::Result<()> {
    let exe = current_exe_str();
    // Create detached session, home window, controller pane (left).
    let status = Command::new("tmux")
        .args([
            "new-session", "-d",
            "-s", target,
            "-n", "home",
            "-c", &repo_root.to_string_lossy(),
            &format!("{exe} __controller"),
        ])
        .status()
        .context("failed to create tmux session")?;
    anyhow::ensure!(status.success(), "tmux new-session failed");

    // Split a right-hand shell pane (default $SHELL) at ~70%.
    let _ = Command::new("tmux")
        .args([
            "split-window", "-h",
            "-t", &format!("{target}:home"),
            "-c", &repo_root.to_string_lossy(),
            "-l", "70%",
        ])
        .status();

    // Select the controller pane (left).
    let _ = Command::new("tmux")
        .args(["select-pane", "-t", &format!("{target}:home.0")])
        .status();

    attach(target)
}

fn attach(target: &str) -> anyhow::Result<()> {
    let status = Command::new("tmux")
        .args(["attach-session", "-t", target])
        .status()
        .context("failed to attach tmux session")?;
    if !status.success() {
        anyhow::bail!("tmux attach exited non-zero");
    }
    Ok(())
}

fn refocus_controller(target: &str) -> anyhow::Result<()> {
    // Pane index 0 in the home window is the controller.
    let _ = Command::new("tmux")
        .args(["select-window", "-t", &format!("{target}:home")])
        .status();
    let _ = Command::new("tmux")
        .args(["select-pane", "-t", &format!("{target}:home.0")])
        .status();
    Ok(())
}

fn print_inside_other_session_error(target: &str, target_exists: bool) {
    eprintln!("wkspace: you're inside a tmux session.");
    if target_exists {
        eprintln!();
        eprintln!("The wkspace session for this repo ('{target}') already exists.");
        eprintln!("Switch to it with:");
        eprintln!("  tmux switch-client -t {target}");
        eprintln!();
        eprintln!("Or detach (Ctrl-B d) and re-run wkspace.");
    } else {
        eprintln!();
        eprintln!("Detach first (Ctrl-B d) and re-run wkspace,");
        eprintln!("or run from a non-tmux terminal.");
    }
}
```

- [ ] **Step 2: Re-export from `src/tui/mod.rs`**

`src/tui/mod.rs`:

```rust
pub mod controller;
pub mod launcher;
pub mod tmux;

pub use launcher::run as run_launcher;
```

- [ ] **Step 3: Verify build**

Run: `cargo build --locked && cargo clippy --locked -- -D warnings`
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add src/tui/launcher.rs src/tui/mod.rs
git commit -m "feat(tui): add launcher orchestration"
```

---

## Task 8: Wire launcher into main.rs no-args path + hidden `__controller` subcommand

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Add hidden `__controller` variant + update no-args branch**

In `src/main.rs`:

Add to the `Commands` enum:

```rust
    /// Internal: run the TUI controller. Hidden — used by the launcher.
    #[command(name = "__controller", hide = true)]
    Controller,
```

Replace the `None => { ... return ... }` block in `main()` with:

```rust
        None => {
            let cwd = std::env::current_dir()?;
            let repo_root = wkspace::git::find_repo_root(&cwd)?;
            return wkspace::tui::run_launcher(&repo_root);
        }
```

Add a match arm for the new variant:

```rust
        Commands::Controller => wkspace::tui::controller::run(),
```

- [ ] **Step 2: Add a stub `controller::run`**

In `src/tui/controller/mod.rs`:

```rust
pub fn run() -> anyhow::Result<()> {
    println!("wkspace controller (stub) — TUI to be implemented");
    // Wait for input so the pane doesn't immediately close in tmux.
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    Ok(())
}
```

- [ ] **Step 3: Verify build**

Run: `cargo build --locked && cargo clippy --locked -- -D warnings`
Expected: clean.

- [ ] **Step 4: Manual smoke test**

(Outside any tmux session, in a wkspace-managed repo:)

```bash
cargo run --
# Expected: tmux session created, controller pane shows "wkspace controller (stub)..."
# Press Enter, the pane closes.
# Press Ctrl-B d to detach, then `tmux kill-session -t wkspace-<slug>` to clean up.
```

(Inside another tmux session:)

```bash
cargo run --
# Expected: stderr message about being inside another tmux session, exit 1.
```

- [ ] **Step 5: Commit**

```bash
git add src/main.rs src/tui/controller/mod.rs
git commit -m "feat(tui): wire launcher to no-args path with hidden controller cmd"
```

---

## Task 9: Controller data layer

Build `Vec<WorktreeRow>` from existing git/list logic, plus tmux session presence.

**Files:**
- Create: `src/tui/controller/data.rs`
- Modify: `src/tui/controller/mod.rs`

- [ ] **Step 1: Create the data module**

`src/tui/controller/data.rs`:

```rust
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
    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let stale_threshold = (ctx.config.worktree.stale_days * 86400) as i64;

    let mut rows = Vec::new();
    for entry in entries.iter().filter(|e| e.path.starts_with(&worktrees_dir)) {
        let name = entry.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
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
```

- [ ] **Step 2: Wire into controller mod**

`src/tui/controller/mod.rs`:

```rust
pub mod data;

pub fn run() -> anyhow::Result<()> {
    println!("wkspace controller (stub) — TUI to be implemented");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    Ok(())
}
```

- [ ] **Step 3: Verify build**

Run: `cargo build --locked && cargo clippy --locked -- -D warnings`
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add src/tui/controller/
git commit -m "feat(tui): add controller data fetch for worktree rows"
```

---

## Task 10: Controller App state struct

**Files:**
- Modify: `src/tui/controller/mod.rs`

- [ ] **Step 1: Add the App state**

Replace `src/tui/controller/mod.rs` with:

```rust
pub mod data;
pub mod events;
pub mod view;

use crate::context;
use anyhow::Context as _;
use std::env;

#[derive(Debug, Default)]
pub enum Modal {
    #[default]
    None,
    Help,
    KillConfirm,
}

#[derive(Debug, Default)]
pub struct Status {
    pub message: Option<String>,
    pub level: StatusLevel,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum StatusLevel {
    #[default]
    Info,
    Warn,
    Error,
}

pub struct App {
    pub repo_root: std::path::PathBuf,
    pub repo_name: String,
    pub base_branch: String,
    pub rows: Vec<data::WorktreeRow>,
    pub selected: usize,
    pub modal: Modal,
    pub status: Status,
    pub should_quit: bool,
    pub kill_session: bool,
}

impl App {
    pub fn new(ctx: &context::Context) -> anyhow::Result<Self> {
        let rows = data::fetch_rows(ctx).unwrap_or_default();
        let repo_name = ctx.repo_root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        Ok(Self {
            repo_root: ctx.repo_root.clone(),
            repo_name,
            base_branch: ctx.config.worktree.base_branch.clone(),
            rows,
            selected: 0,
            modal: Modal::None,
            status: Status::default(),
            should_quit: false,
            kill_session: false,
        })
    }

    pub fn refresh(&mut self, ctx: &context::Context) {
        match data::fetch_rows(ctx) {
            Ok(r) => {
                self.rows = r;
                if self.selected >= self.rows.len() && !self.rows.is_empty() {
                    self.selected = self.rows.len() - 1;
                }
            }
            Err(e) => {
                self.status = Status {
                    message: Some(format!("refresh failed: {e}")),
                    level: StatusLevel::Error,
                };
            }
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    let cwd = env::current_dir().context("get cwd")?;
    let ctx = context::resolve(&cwd)?;
    let mut app = App::new(&ctx)?;
    // Real run loop comes in Task 16.
    let _ = &mut app;
    println!("controller boot ok ({} rows)", app.rows.len());
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    Ok(())
}
```

- [ ] **Step 2: Create empty stub files for events + view**

`src/tui/controller/events.rs`:

```rust
// Pure key→action mapping lives here.
```

`src/tui/controller/view.rs`:

```rust
// ratatui rendering lives here.
```

- [ ] **Step 3: Verify build**

Run: `cargo build --locked && cargo clippy --locked -- -D warnings`
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add src/tui/controller/
git commit -m "feat(tui): add controller App state struct"
```

---

## Task 11: View — render scaffolding (title bar + worktree list)

**Files:**
- Modify: `src/tui/controller/view.rs`
- Create: `tests/tui_view_test.rs`

- [ ] **Step 1: Write a snapshot test for the title bar**

`tests/tui_view_test.rs`:

```rust
use ratatui::{backend::TestBackend, Terminal};
use wkspace::tui::controller::{data::WorktreeRow, view, App, Modal, Status};

fn empty_app() -> App {
    App {
        repo_root: "/tmp/foo".into(),
        repo_name: "foo".into(),
        base_branch: "main".into(),
        rows: vec![],
        selected: 0,
        modal: Modal::None,
        status: Status::default(),
        should_quit: false,
        kill_session: false,
    }
}

fn one_row(name: &str) -> WorktreeRow {
    WorktreeRow {
        name: name.into(),
        branch: name.into(),
        uncommitted: 0,
        last_commit: "1 hour ago".into(),
        stale: false,
        description: "".into(),
        has_session: false,
    }
}

#[test]
fn renders_title_bar_with_repo_name_and_count() {
    let backend = TestBackend::new(30, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = empty_app();
    terminal.draw(|f| view::render(f, &app)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let line0: String = (0..30).map(|x| buffer[(x, 0)].symbol().to_string()).collect();
    assert!(line0.contains("wkspace"));
    assert!(line0.contains("foo"));
}

#[test]
fn empty_state_shows_hint() {
    let backend = TestBackend::new(40, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = empty_app();
    terminal.draw(|f| view::render(f, &app)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let dump: String = (0..20)
        .flat_map(|y| (0..40).map(move |x| buffer[(x, y)].symbol().to_string()))
        .collect();
    assert!(dump.contains("No worktrees"));
}

#[test]
fn renders_row_names() {
    let backend = TestBackend::new(40, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = empty_app();
    app.rows = vec![one_row("alpha"), one_row("beta")];
    terminal.draw(|f| view::render(f, &app)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let dump: String = (0..20)
        .flat_map(|y| (0..40).map(move |x| buffer[(x, y)].symbol().to_string()))
        .collect();
    assert!(dump.contains("alpha"));
    assert!(dump.contains("beta"));
}
```

- [ ] **Step 2: Run, verify fail**

Run: `cargo test --test tui_view_test --locked`
Expected: fails — `view::render` doesn't exist; some types may be private.

- [ ] **Step 3: Make state types `pub` if not already**

(They already are per Task 10 — verify quickly.)

- [ ] **Step 4: Implement layout sketch**

Replace `src/tui/controller/view.rs`:

```rust
use crate::tui::controller::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // title bar
            Constraint::Min(3),     // worktree list
            Constraint::Length(7),  // detail block
            Constraint::Length(3),  // keymap footer
            Constraint::Length(1),  // status line
        ])
        .split(area);

    render_title(f, chunks[0], app);
    render_list(f, chunks[1], app);
    render_detail(f, chunks[2], app);
    render_footer(f, chunks[3], app);
    render_status(f, chunks[4], app);
}

fn render_title(f: &mut Frame, area: Rect, app: &App) {
    let line1 = format!("wkspace · {}", app.repo_name);
    let count = app.rows.len();
    let line2 = format!("{} · {} worktrees", app.base_branch, count);
    let p = Paragraph::new(vec![
        Line::from(Span::styled(line1, Style::default().add_modifier(Modifier::BOLD))),
        Line::from(line2),
    ]);
    f.render_widget(p, area);
}

fn render_list(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::TOP);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.rows.is_empty() {
        let hint = Paragraph::new(vec![
            Line::from(""),
            Line::from("No worktrees yet."),
            Line::from(""),
            Line::from("Press 'n' to create one"),
            Line::from("or 'f' for an existing branch."),
        ]);
        f.render_widget(hint, inner);
        return;
    }

    let lines: Vec<Line> = app.rows.iter().enumerate().map(|(i, row)| {
        let marker = if i == app.selected { "▸ " } else { "  " };
        let session = if row.has_session { " ●" } else { "  " };
        let dirty = if row.uncommitted > 0 { " !" } else { "  " };
        let stale = if row.stale { " ·" } else { "  " };
        let style = if i == app.selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        Line::from(Span::styled(
            format!("{marker}{:<20}{session}{dirty}{stale}", row.name),
            style,
        ))
    }).collect();

    let p = Paragraph::new(lines);
    f.render_widget(p, inner);
}

fn render_detail(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::TOP);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(row) = app.rows.get(app.selected) else {
        return;
    };
    let p = Paragraph::new(vec![
        Line::from(Span::styled(&row.name, Style::default().add_modifier(Modifier::BOLD))),
        Line::from(format!("branch:  {}", row.branch)),
        Line::from(format!("commit:  {}", row.last_commit)),
        Line::from(format!(
            "status:  {}",
            if row.uncommitted > 0 {
                format!("{} uncommitted", row.uncommitted)
            } else {
                "clean".into()
            }
        )),
        Line::from(format!("session: {}", if row.has_session { "active" } else { "—" })),
        Line::from(format!("desc:    {}", row.description)),
    ]);
    f.render_widget(p, inner);
}

fn render_footer(f: &mut Frame, area: Rect, _app: &App) {
    let block = Block::default().borders(Borders::TOP);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let p = Paragraph::new(vec![
        Line::from("n new   f from   o open"),
        Line::from("d rm    s setup  t teardown"),
        Line::from("?  help    q  quit/detach"),
    ]);
    f.render_widget(p, inner);
}

fn render_status(f: &mut Frame, area: Rect, app: &App) {
    let msg = app.status.message.clone().unwrap_or_default();
    let p = Paragraph::new(Line::from(msg));
    f.render_widget(p, area);
}
```

- [ ] **Step 5: Run, verify pass**

Run: `cargo test --test tui_view_test --locked`
Expected: 3 passed.

- [ ] **Step 6: Commit**

```bash
git add src/tui/controller/view.rs tests/tui_view_test.rs
git commit -m "feat(tui): render controller layout"
```

---

## Task 12: Help overlay rendering

**Files:**
- Modify: `src/tui/controller/view.rs`
- Modify: `tests/tui_view_test.rs`

- [ ] **Step 1: Add a test for help overlay**

Append to `tests/tui_view_test.rs`:

```rust
#[test]
fn help_overlay_renders_when_active() {
    let backend = TestBackend::new(60, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = empty_app();
    app.modal = Modal::Help;
    terminal.draw(|f| view::render(f, &app)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let dump: String = (0..30)
        .flat_map(|y| (0..60).map(move |x| buffer[(x, y)].symbol().to_string()))
        .collect();
    assert!(dump.contains("Help"));
    assert!(dump.contains("navigate"));
}
```

- [ ] **Step 2: Run, verify fail**

- [ ] **Step 3: Render help overlay over base layout**

In `src/tui/controller/view.rs`, append to `render`:

```rust
    if matches!(app.modal, crate::tui::controller::Modal::Help) {
        render_help_overlay(f, area);
    }
    if matches!(app.modal, crate::tui::controller::Modal::KillConfirm) {
        render_kill_confirm(f, area);
    }
```

And add helpers:

```rust
fn render_help_overlay(f: &mut Frame, area: Rect) {
    let block = Block::default().title(" Help ").borders(Borders::ALL);
    let lines = vec![
        Line::from(""),
        Line::from("  ↑/↓ k/j      navigate"),
        Line::from("  g  G         top / bottom"),
        Line::from("  enter / o    open or focus session"),
        Line::from("  n            new worktree (M4)"),
        Line::from("  f            from existing branch (M4)"),
        Line::from("  d            remove worktree (M4)"),
        Line::from("  s  t         re-run setup / teardown (M4)"),
        Line::from("  r            refresh"),
        Line::from("  ?            toggle help"),
        Line::from("  q  Ctrl-C    detach session"),
        Line::from("  Q            kill session (with confirm)"),
        Line::from(""),
        Line::from("  press ? or esc to close"),
    ];
    let p = Paragraph::new(lines).block(block);
    let overlay = centered_rect(50, 18, area);
    f.render_widget(ratatui::widgets::Clear, overlay);
    f.render_widget(p, overlay);
}

fn render_kill_confirm(f: &mut Frame, area: Rect) {
    let block = Block::default().title(" Kill session? ").borders(Borders::ALL);
    let p = Paragraph::new(vec![
        Line::from(""),
        Line::from("  This will close all worktree windows in this repo's"),
        Line::from("  wkspace session. Long-running processes (Claude,"),
        Line::from("  dev servers) will be terminated."),
        Line::from(""),
        Line::from("  Press y to confirm, n or esc to cancel."),
    ]).block(block);
    let overlay = centered_rect(60, 8, area);
    f.render_widget(ratatui::widgets::Clear, overlay);
    f.render_widget(p, overlay);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect { x, y, width: width.min(area.width), height: height.min(area.height) }
}
```

- [ ] **Step 4: Run, verify pass**

- [ ] **Step 5: Commit**

```bash
git add src/tui/controller/view.rs tests/tui_view_test.rs
git commit -m "feat(tui): render help and kill-session overlays"
```

---

## Task 13: Event handler — pure key→action mapping

**Files:**
- Modify: `src/tui/controller/events.rs`
- Create: `tests/tui_events_test.rs`

- [ ] **Step 1: Write tests**

`tests/tui_events_test.rs`:

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use wkspace::tui::controller::{events::{handle_key, Action}, App, Modal, Status, data::WorktreeRow};

fn app_with_rows(n: usize) -> App {
    App {
        repo_root: "/tmp/foo".into(),
        repo_name: "foo".into(),
        base_branch: "main".into(),
        rows: (0..n).map(|i| WorktreeRow {
            name: format!("w{i}"), branch: format!("w{i}"),
            uncommitted: 0, last_commit: "—".into(), stale: false,
            description: "".into(), has_session: false,
        }).collect(),
        selected: 0,
        modal: Modal::None,
        status: Status::default(),
        should_quit: false,
        kill_session: false,
    }
}

fn key(c: char) -> KeyEvent { KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE) }

#[test]
fn down_increments_selection() {
    let mut app = app_with_rows(3);
    handle_key(&mut app, key('j'));
    assert_eq!(app.selected, 1);
}

#[test]
fn up_at_top_clamps() {
    let mut app = app_with_rows(3);
    handle_key(&mut app, key('k'));
    assert_eq!(app.selected, 0);
}

#[test]
fn down_at_bottom_clamps() {
    let mut app = app_with_rows(2);
    app.selected = 1;
    handle_key(&mut app, key('j'));
    assert_eq!(app.selected, 1);
}

#[test]
fn g_jumps_to_top() {
    let mut app = app_with_rows(5);
    app.selected = 4;
    handle_key(&mut app, key('g'));
    assert_eq!(app.selected, 0);
}

#[test]
fn capital_g_jumps_to_bottom() {
    let mut app = app_with_rows(5);
    handle_key(&mut app, KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT));
    assert_eq!(app.selected, 4);
}

#[test]
fn q_sets_should_quit() {
    let mut app = app_with_rows(1);
    handle_key(&mut app, key('q'));
    assert!(app.should_quit);
}

#[test]
fn ctrl_c_quits() {
    let mut app = app_with_rows(1);
    handle_key(&mut app, KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert!(app.should_quit);
}

#[test]
fn question_mark_toggles_help() {
    let mut app = app_with_rows(1);
    handle_key(&mut app, key('?'));
    assert!(matches!(app.modal, Modal::Help));
    handle_key(&mut app, key('?'));
    assert!(matches!(app.modal, Modal::None));
}

#[test]
fn esc_dismisses_help() {
    let mut app = app_with_rows(1);
    app.modal = Modal::Help;
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(matches!(app.modal, Modal::None));
}

#[test]
fn capital_q_opens_confirm() {
    let mut app = app_with_rows(1);
    handle_key(&mut app, KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::SHIFT));
    assert!(matches!(app.modal, Modal::KillConfirm));
}

#[test]
fn y_in_kill_confirm_sets_kill_flag() {
    let mut app = app_with_rows(1);
    app.modal = Modal::KillConfirm;
    handle_key(&mut app, key('y'));
    assert!(app.kill_session);
    assert!(app.should_quit);
}

#[test]
fn n_in_kill_confirm_cancels() {
    let mut app = app_with_rows(1);
    app.modal = Modal::KillConfirm;
    handle_key(&mut app, key('n'));
    assert!(!app.kill_session);
    assert!(matches!(app.modal, Modal::None));
}

#[test]
fn refresh_action_returned() {
    let mut app = app_with_rows(1);
    let action = handle_key(&mut app, key('r'));
    assert_eq!(action, Action::Refresh);
}

#[test]
fn other_keys_do_nothing() {
    let mut app = app_with_rows(1);
    let before = app.selected;
    handle_key(&mut app, key('z'));
    assert_eq!(app.selected, before);
    assert!(!app.should_quit);
}
```

- [ ] **Step 2: Run, verify fail**

- [ ] **Step 3: Implement**

Replace `src/tui/controller/events.rs`:

```rust
use crate::tui::controller::{App, Modal};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Refresh,
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    if matches!(app.modal, Modal::Help) {
        return handle_help(app, key);
    }
    if matches!(app.modal, Modal::KillConfirm) {
        return handle_kill_confirm(app, key);
    }

    match (key.code, key.modifiers) {
        (KeyCode::Char('j') | KeyCode::Down, _) => move_down(app),
        (KeyCode::Char('k') | KeyCode::Up, _) => move_up(app),
        (KeyCode::Char('g'), m) if !m.contains(KeyModifiers::SHIFT) => app.selected = 0,
        (KeyCode::Char('G'), _) => {
            if !app.rows.is_empty() {
                app.selected = app.rows.len() - 1;
            }
        }
        (KeyCode::Char('q'), _) => app.should_quit = true,
        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => app.should_quit = true,
        (KeyCode::Char('Q'), _) => app.modal = Modal::KillConfirm,
        (KeyCode::Char('?'), _) => app.modal = Modal::Help,
        (KeyCode::Char('r'), _) => return Action::Refresh,
        _ => {}
    }
    Action::None
}

fn move_down(app: &mut App) {
    if !app.rows.is_empty() && app.selected + 1 < app.rows.len() {
        app.selected += 1;
    }
}

fn move_up(app: &mut App) {
    if app.selected > 0 {
        app.selected -= 1;
    }
}

fn handle_help(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('?') | KeyCode::Esc => app.modal = Modal::None,
        _ => {}
    }
    Action::None
}

fn handle_kill_confirm(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.kill_session = true;
            app.should_quit = true;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.modal = Modal::None;
        }
        _ => {}
    }
    Action::None
}
```

- [ ] **Step 4: Run, verify pass**

Run: `cargo test --test tui_events_test --locked`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src/tui/controller/events.rs tests/tui_events_test.rs
git commit -m "feat(tui): add pure key event handler"
```

---

## Task 14: Controller event loop + terminal lifecycle

Wire ratatui+crossterm: enter alt-screen, raw mode, restore on panic. Tick at 2s for refresh.

**Files:**
- Modify: `src/tui/controller/mod.rs`

- [ ] **Step 1: Replace stub `run()` with real loop**

In `src/tui/controller/mod.rs`, replace `run()`:

```rust
pub fn run() -> anyhow::Result<()> {
    use crossterm::{
        event::{self, Event, EnableMouseCapture, DisableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::io::stdout;
    use std::time::{Duration, Instant};

    let cwd = env::current_dir().context("get cwd")?;
    let ctx = context::resolve(&cwd)?;
    let mut app = App::new(&ctx)?;

    // Restore terminal on panic.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(info);
    }));

    enable_raw_mode().context("enable raw mode")?;
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture).context("enter alt screen")?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).context("create terminal")?;

    let tick = Duration::from_secs(2);
    let mut last_tick = Instant::now();

    let result = (|| -> anyhow::Result<()> {
        loop {
            terminal.draw(|f| view::render(f, &app))?;

            let timeout = tick.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(k) = event::read()? {
                    let action = events::handle_key(&mut app, k);
                    if action == events::Action::Refresh {
                        app.refresh(&ctx);
                    }
                }
            }
            if last_tick.elapsed() >= tick {
                app.refresh(&ctx);
                last_tick = Instant::now();
            }
            if app.should_quit {
                break;
            }
        }
        Ok(())
    })();

    disable_raw_mode().ok();
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture).ok();
    terminal.show_cursor().ok();

    if app.kill_session {
        let target = crate::tui::tmux::session_name(&ctx.repo_root);
        let _ = std::process::Command::new("tmux")
            .args(["kill-session", "-t", &target])
            .status();
    } else {
        // Detach from the current client (run when in tmux). No-op outside tmux.
        let _ = std::process::Command::new("tmux").arg("detach-client").status();
    }

    result
}
```

- [ ] **Step 2: Verify build**

Run: `cargo build --locked && cargo clippy --locked -- -D warnings`
Expected: clean.

- [ ] **Step 3: Verify all tests still pass**

Run: `cargo test --locked`
Expected: clean.

- [ ] **Step 4: Manual smoke test**

(In a wkspace-managed repo, outside tmux:)

```bash
cargo run --
```

Expected:
- tmux session created
- Sidebar shows the layout: title bar, list (or "No worktrees"), detail block, keymap footer.
- `j`/`k` move selection.
- `?` opens help overlay; `?` or `esc` closes it.
- `Q` opens kill-confirm; `n`/`esc` cancels; `y` ends and kills the session.
- `q` detaches; re-running `wkspace` re-attaches with the same state.

(Inside the wkspace session, in another pane:)

```bash
wkspace
# Expected: refocuses controller pane, no new processes.
```

- [ ] **Step 5: Commit**

```bash
git add src/tui/controller/mod.rs
git commit -m "feat(tui): wire controller event loop with refresh tick"
```

---

## Task 15: Final polish + full CI

- [ ] **Step 1: Run the full CI gate**

```bash
cargo test --locked && cargo clippy --locked -- -D warnings && cargo fmt --check
```

Expected: clean.

- [ ] **Step 2: Manual end-to-end verification**

In the test repo, verify:
- Cold launch outside tmux creates the session and attaches.
- `q` detaches. Re-launch attaches to existing session.
- Inside *another* tmux session: error message appears, exit 1.
- Inside the wkspace session for this repo: refocus path runs, no new processes.
- Help overlay (`?`) toggles cleanly.
- Kill-session (`Q` → `y`) tears the session down.
- Resizing the terminal redraws cleanly.
- The right pane is still a usable shell.

- [ ] **Step 3: Update README with a brief TUI section**

Add a small note to `README.md` (full rewrite is M5):

```markdown
## TUI mode (preview)

Running `wkspace` with no arguments launches a tmux-backed TUI showing
your worktrees alongside a regular shell pane. Navigate with arrow keys
or `j`/`k`, press `?` for help, `q` to detach. Requires tmux ≥ 3.0.

Mutating actions (`n`/`f`/`d`/`s`/`t`) are not yet wired up — use the
existing CLI subcommands for those. Full TUI parity lands in upcoming
milestones.
```

- [ ] **Step 4: Final commit**

```bash
git add README.md
git commit -m "docs: note TUI preview in README"
```

---

## Self-review

Spec coverage check (against `2026-05-04-tui-redesign-design.md` M1):

| Spec requirement | Tasks |
|---|---|
| `ratatui` + `crossterm` deps | Task 1 |
| tmux preflight | Task 4 |
| Hidden `__controller` subcommand | Task 8 |
| Tmux session detect/create + attach | Tasks 5–8 |
| Three boot scenarios | Tasks 6, 7, 8 |
| Error path inside another session | Tasks 6, 7 |
| Layout: title, list, detail, footer, status | Tasks 11, 12 |
| Navigation: `↑/↓/j/k/g/G` | Task 13 |
| Refresh: `r` | Tasks 13, 14 |
| Help overlay: `?` | Tasks 12, 13 |
| Quit/detach: `q`, Ctrl-C | Tasks 13, 14 |
| Kill session: `Q` w/ confirm | Tasks 12, 13, 14 |
| No mutating actions yet | (covered by absence) |
| Read worktree state from existing git logic | Task 9 |
| Tick-based refresh | Task 14 |

All M1 DoD items are covered.

---

## Out of scope for M1 (per spec)

- `o` and `enter` actions, `●` session marker → M2
- Refactoring `commands::*::run` into pure functions → M3
- `n`, `f`, `d`, `s`, `t` actions → M4
- `Q` confirmation polish, first-run config, README rewrite → M5

These get their own plans.
