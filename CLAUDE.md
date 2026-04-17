# Project: wkspace

Rust CLI tool for managing Git worktrees with lifecycle scripts.

## Workflow

When starting any task, ensure main is up to date before branching:

```sh
git fetch --all
git pull origin main --ff-only
```

Then create a worktree:

```sh
wkspace new <branch-name> --no-shell
```

Work in `.worktrees/<branch-name>/`.

When a piece of work is finished, offer the user a chance to manually test the changes before committing. Once approved, commit and push. For anything user-visible, create a PR. Clean up with `wkspace rm <branch-name>` once merged.

After a new feature or bugfix is implemented, offer to create a new release.

## After changing command behaviour

Update `README.md` to reflect any new flags, changed steps, or removed features.

## After making changes

Always run the full CI check locally before considering work complete:

```sh
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

If `cargo fmt --check` fails, run `cargo fmt` to fix formatting.
