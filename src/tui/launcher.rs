use crate::tui::tmux::TmuxEnv;

pub struct Inputs {
    pub env: TmuxEnv,
    pub is_target: bool,
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
            if inputs.is_target {
                Decision::RefocusController
            } else {
                Decision::ErrorInsideOtherSession {
                    target_exists: inputs.target_session_exists,
                }
            }
        }
    }
}

// Orchestrator: reads env, queries tmux, calls decide(), executes the action.

use crate::tui::tmux;
use anyhow::Context;
use std::path::Path;
use std::process::Command;

pub fn run(repo_root: &Path) -> anyhow::Result<()> {
    tmux::preflight().context("tmux preflight failed")?;

    let target = tmux::session_name(repo_root);
    let env = tmux::detect_env(&|k| std::env::var(k).ok());
    let current = if env == tmux::TmuxEnv::Inside {
        tmux::current_session_name()
    } else {
        None
    };
    let exists = tmux::session_exists(&target);
    let is_target = current.as_deref() == Some(target.as_str());

    let decision = decide(Inputs {
        env,
        is_target,
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
            "new-session",
            "-d",
            "-s",
            target,
            "-n",
            "home",
            "-c",
            &repo_root.to_string_lossy(),
            &format!("{exe} __controller"),
        ])
        .status()
        .context("failed to create tmux session")?;
    anyhow::ensure!(status.success(), "tmux new-session failed");

    // Split a right-hand shell pane (default $SHELL) at ~70%.
    let _ = Command::new("tmux")
        .args([
            "split-window",
            "-h",
            "-t",
            &format!("{target}:home"),
            "-c",
            &repo_root.to_string_lossy(),
            "-l",
            "70%",
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
