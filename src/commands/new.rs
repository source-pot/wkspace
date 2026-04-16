use crate::context;
use crate::error::WkspaceError;
use crate::git;
use crate::hooks;
use crate::ports;
use crate::scripts;
use dialoguer::Input;
use std::collections::HashMap;
use std::env;
use std::process::Command;

pub fn prompt_name() -> anyhow::Result<String> {
    let name: String = Input::new().with_prompt("Worktree name").interact_text()?;
    Ok(name)
}

pub fn run(name: &str, desc: Option<&str>, no_shell: bool, no_scripts: bool) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let ctx = context::resolve(&cwd)?;

    // Apply prefix to branch name if configured
    let prefix = &ctx.config.worktree.prefix;
    let branch_name = if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}/{name}")
    };
    let worktree_name = branch_name.replace('/', "-");
    let worktree_path = ctx.worktree_path(&worktree_name);

    // Check if worktree directory already exists
    if worktree_path.exists() {
        anyhow::bail!(WkspaceError::WorktreeExists(worktree_name.clone()));
    }

    // Allocate ports before worktree creation (fail early)
    let port_env = ports::allocate_ports(&ctx.config.ports)?;
    if !port_env.is_empty() {
        println!("Allocated ports:");
        for (var, port) in &port_env {
            println!("  {var}={port}");
        }
    }

    // Build script environment: ports + worktree metadata
    let mut script_env = port_env;
    script_env.insert("WORKTREE_NAME".to_string(), worktree_name.clone());

    // Fetch latest from remote before branching
    println!("Fetching from remote...");
    let remote = &ctx.config.worktree.remote;
    git::fetch_all(&ctx.repo_root, remote);

    // Prefer remote base branch; fall back to local if no remote exists
    let remote_base = format!("{remote}/{}", ctx.config.worktree.base_branch);
    let start_point = if git::ref_exists(&ctx.repo_root, &remote_base) {
        &remote_base
    } else {
        &ctx.config.worktree.base_branch
    };
    println!("Creating worktree '{worktree_name}' from '{start_point}'...");
    git::add_worktree(&ctx.repo_root, &worktree_path, &branch_name, start_point)?;

    // Store branch description if provided
    if let Some(d) = desc {
        git::set_branch_description(&ctx.repo_root, &branch_name, d)?;
    }

    // Run setup scripts
    if !no_scripts && !ctx.config.scripts.setup.is_empty() {
        println!("Running setup scripts...");
        scripts::run_scripts(&ctx.config.scripts.setup, &worktree_path, &script_env)?;
    }

    // Run user hook
    hooks::run_hook("post-new", &worktree_path, &script_env, None);

    // Spawn subshell (skip via --no-shell flag or WKSPACE_NO_SHELL env var)
    if !no_shell && env::var("WKSPACE_NO_SHELL").is_err() {
        spawn_shell(&worktree_path, &script_env)?;
    }

    Ok(())
}

pub(crate) fn spawn_shell(
    cwd: &std::path::Path,
    extra_env: &HashMap<String, String>,
) -> anyhow::Result<()> {
    let shell = env::var("WKSPACE_SHELL")
        .or_else(|_| env::var("SHELL"))
        .unwrap_or_else(|_| "/bin/sh".to_string());
    println!("Opening shell in {}...", cwd.display());
    let mut child = Command::new(&shell)
        .current_dir(cwd)
        .envs(extra_env)
        .spawn()?;
    child.wait()?;
    Ok(())
}
