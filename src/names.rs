use crate::context;
use crate::git;
use rand::Rng;
use std::env;

const MAX_ATTEMPTS: u32 = 50;

/// Generate a random 8-character hex string.
fn generate_hex_name(rng: &mut impl Rng) -> String {
    let bytes: [u8; 4] = rng.gen();
    hex_encode(&bytes)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Generate a unique worktree name that doesn't collide with existing
/// worktrees or git branches.
pub fn generate_unique_name() -> anyhow::Result<String> {
    let cwd = env::current_dir()?;
    let ctx = context::resolve(&cwd)?;
    let worktrees_dir = ctx.worktrees_dir();
    let mut rng = rand::thread_rng();

    for _ in 0..MAX_ATTEMPTS {
        let name = generate_hex_name(&mut rng);
        if worktrees_dir.join(&name).exists() {
            continue;
        }
        if git::branch_exists(&ctx.repo_root, &name)? {
            continue;
        }
        return Ok(name);
    }

    anyhow::bail!("Failed to generate a unique name after {MAX_ATTEMPTS} attempts")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_hex_name_is_8_hex_chars() {
        let mut rng = rand::thread_rng();
        let name = generate_hex_name(&mut rng);
        assert_eq!(name.len(), 8);
        assert!(name.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
