use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "wkspace",
    about = "Manage Git worktrees with lifecycle scripts",
    long_about = "\
Manage Git worktrees with lifecycle scripts.

wkspace creates isolated Git worktrees in a .worktrees/ directory, each on its
own branch. It runs configurable setup and teardown scripts automatically, and
can allocate random ports exposed as environment variables.

Configuration lives in .wkspace.toml at the repository root. Run `wkspace init`
to create one with documented defaults.

Typical workflow:
  wkspace new my-feature      Create worktree, run setup, open shell
  wkspace list                See all active worktrees
  wkspace rm my-feature       Run teardown and clean up

Worktrees are stored in <directory>/<name>/ (default: .worktrees/<name>/).
Branch names can be auto-prefixed (e.g. \"rob/my-feature\") via the prefix
config option.",
    after_help = "Use --help (not -h) for extended information about each command.",
    after_long_help = "\
ENVIRONMENT VARIABLES:
  WKSPACE_SHELL     Shell to spawn in worktrees (falls back to $SHELL, then /bin/sh)
  WKSPACE_NO_SHELL  If set, skip spawning a shell after new/from/open

CONFIGURATION (.wkspace.toml):
  [worktree]
    base_branch   Branch that new worktrees are based on (default: \"main\")
    directory     Where worktrees are stored (default: \".worktrees\")
    stale_days    Days before a worktree is marked stale (default: 7)
    prefix        Auto-prefix for branch names (e.g. \"rob\" -> \"rob/name\")
    remote        Git remote name (default: \"origin\")

  [scripts]
    setup         Commands run after creating a worktree
    teardown      Commands run before removing a worktree

  [ports]
    <label> = \"ENV_VAR\"   Allocate random ports exposed to scripts and shell

USER HOOKS:
  Per-user scripts in ~/.config/wkspace/hooks/ run after each command:
    post-init, post-new, post-from, post-rm, post-open,
    post-setup, post-teardown, post-list

  Scripts receive WORKTREE_NAME and any allocated port variables."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create .wkspace.toml with default configuration
    #[command(long_about = "\
Create .wkspace.toml with default configuration.

Creates a .wkspace.toml file at the repository root with commented defaults for
all options. Also adds .worktrees to .gitignore if not already present.

Safe to run in a repo that already has a config — it will not overwrite an
existing .wkspace.toml.")]
    Init,

    /// Create a new worktree with a branch, run setup scripts, and open a shell
    #[command(long_about = "\
Create a new worktree with a branch, run setup scripts, and open a shell.

Steps performed:
  1. Allocate random ports (if configured in [ports])
  2. Fetch latest from the configured remote
  3. Create a new branch from the remote base branch (e.g. origin/main)
  4. Add a Git worktree at .worktrees/<name>/
  5. Store the branch description (if --desc provided)
  6. Run setup scripts from .wkspace.toml (unless --no-scripts)
  7. Run the post-new user hook
  8. Spawn an interactive shell in the worktree (unless --no-shell)

If a branch prefix is configured (e.g. \"rob\"), the branch is created as
\"rob/<name>\" and the worktree directory is named \"rob-<name>\".

The WORKTREE_NAME and any allocated port variables are available as
environment variables in scripts and the spawned shell.")]
    New {
        /// Name for the worktree and branch (prompted if omitted)
        name: Option<String>,
        /// Description for the worktree branch
        #[arg(short, long)]
        desc: Option<String>,
        /// Skip spawning an interactive shell after setup
        #[arg(long)]
        no_shell: bool,
        /// Skip running lifecycle scripts
        #[arg(long)]
        no_scripts: bool,
    },

    /// Create a worktree from an existing branch
    #[command(long_about = "\
Create a worktree from an existing remote or local branch.

Like `new`, but checks out an existing branch instead of creating one. Useful
for resuming work on a branch that was pushed from another machine.

Steps performed:
  1. Allocate random ports (if configured in [ports])
  2. Fetch latest from the configured remote
  3. Fast-forward the local branch to match the remote
  4. Add a Git worktree at .worktrees/<name>/
  5. Run setup scripts from .wkspace.toml (unless --no-scripts)
  6. Run the post-from user hook
  7. Spawn an interactive shell in the worktree

Cannot be used with the base branch — use `wkspace new` instead.
If the branch name contains slashes (e.g. \"feat/login\"), the worktree
directory uses dashes instead (e.g. \"feat-login\").")]
    From {
        /// Branch name (interactive picker if omitted)
        branch: Option<String>,
        /// Skip running lifecycle scripts
        #[arg(long)]
        no_scripts: bool,
    },

    /// Run teardown scripts and remove a worktree and its branch
    #[command(long_about = "\
Run teardown scripts and remove a worktree and its branch.

Steps performed:
  1. Check for uncommitted changes (prompts for confirmation unless --force)
  2. Run teardown scripts from .wkspace.toml (unless --no-scripts)
  3. Remove the Git worktree directory and metadata
  4. Delete the local branch
  5. Run the post-rm user hook

The worktree directory and its contents are permanently deleted. Any unpushed
commits on the branch will be lost.")]
    Rm {
        /// Name of the worktree to remove (interactive picker if omitted)
        name: Option<String>,
        /// Skip confirmation prompt for uncommitted changes
        #[arg(short, long)]
        force: bool,
        /// Skip running lifecycle scripts
        #[arg(long)]
        no_scripts: bool,
    },

    /// List active worktrees
    #[command(long_about = "\
List active worktrees.

Shows a table of all worktrees managed by wkspace (those inside the configured
worktrees directory). Columns:

  NAME         Worktree directory name
  STATUS       \"clean\" or number of uncommitted changes
  LAST COMMIT  Relative time since the last commit on the branch
  DESCRIPTION  Branch description, with \"stale\" marker if applicable

A worktree is marked stale when its last commit is older than the configured
stale_days threshold (default: 7 days).")]
    List,

    /// Open a shell in an existing worktree
    #[command(long_about = "\
Open a shell in an existing worktree.

Spawns an interactive shell with the working directory set to the worktree.
Does not re-run setup scripts — use `wkspace setup` for that.

The shell used is determined by WKSPACE_SHELL, then $SHELL, then /bin/sh.")]
    Open {
        /// Name of the worktree to open (interactive picker if omitted)
        name: Option<String>,
    },

    /// Re-run setup scripts in the current worktree
    #[command(long_about = "\
Re-run setup scripts in the current worktree.

Must be run from inside a worktree directory. Re-executes all commands listed
in [scripts].setup from .wkspace.toml, with freshly allocated ports.

Useful after pulling changes that modify project dependencies or configuration.")]
    Setup,

    /// Re-run teardown scripts in the current worktree
    #[command(long_about = "\
Re-run teardown scripts in the current worktree.

Must be run from inside a worktree directory. Re-executes all commands listed
in [scripts].teardown from .wkspace.toml.

Useful for cleaning up resources (e.g. stopping services) without removing the
worktree itself.")]
    Teardown,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            Cli::parse_from(["wkspace", "--help"]);
            return Ok(());
        }
    };

    match command {
        Commands::Init => wkspace::commands::init::run(),
        Commands::New {
            name,
            desc,
            no_shell,
            no_scripts,
        } => {
            let name = match name {
                Some(n) => n,
                None => wkspace::commands::new::prompt_name()?,
            };
            wkspace::commands::new::run(&name, desc.as_deref(), no_shell, no_scripts)
        }
        Commands::From { branch, no_scripts } => {
            let branch = match branch {
                Some(b) => b,
                None => wkspace::commands::from::pick_branch()?,
            };
            wkspace::commands::from::run(&branch, no_scripts)
        }
        Commands::Rm {
            name,
            force,
            no_scripts,
        } => {
            let name = match name {
                Some(n) => n,
                None => wkspace::commands::pick_worktree("Select worktree to remove")?,
            };
            wkspace::commands::rm::run(&name, force, no_scripts)
        }
        Commands::List => wkspace::commands::list::run(),
        Commands::Open { name } => {
            let name = match name {
                Some(n) => n,
                None => wkspace::commands::pick_worktree("Select worktree to open")?,
            };
            wkspace::commands::open::run(&name)
        }
        Commands::Setup => wkspace::commands::setup::run(),
        Commands::Teardown => wkspace::commands::teardown::run(),
    }
}
