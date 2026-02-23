use crate::error::WkspaceError;
use std::path::Path;
use std::process::Command;

/// Run a list of shell commands sequentially in the given directory.
/// Stops on first failure.
pub fn run_scripts(commands: &[String], cwd: &Path) -> anyhow::Result<()> {
    for cmd in commands {
        println!("  Running: {cmd}");
        let status = Command::new("sh")
            .args(["-c", cmd])
            .current_dir(cwd)
            .status()?;

        if !status.success() {
            anyhow::bail!(WkspaceError::ScriptFailed {
                command: cmd.clone(),
                exit_code: status.code(),
            });
        }
    }
    Ok(())
}
