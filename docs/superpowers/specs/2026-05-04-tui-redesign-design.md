# wkspace TUI Redesign — Design

**Date:** 2026-05-04
**Status:** Design approved, awaiting implementation plan

## Goals

1. **Single entry point.** Running `wkspace` with no args should expose every feature of the tool. Users shouldn't need to remember subcommand names like `from` or `open`.
2. **Discoverability.** The available actions should be visible on screen at all times.
3. **Visually engaging.** Feel like an application, not a sequence of CLI prompts.
4. **State preservation across worktrees.** Long-running processes (Claude sessions, dev servers, REPLs, watch processes) must survive when the user switches focus to work in a different worktree.
5. **Backwards compatible.** The existing CLI subcommands (`wkspace new foo`, `wkspace list`, etc.) keep working unchanged. The TUI is purely additive: new behaviour for the no-args case.

## Non-goals

- Building a terminal multiplexer ourselves. tmux already does this well; we use it as a substrate.
- Replacing the existing CLI surface. Scripts, hooks, and muscle memory remain valid.
- Cross-platform native Windows support beyond what tmux offers (i.e. WSL is fine, native Windows isn't).
- A custom theme system. Sensible terminal-respecting defaults only.

## Architecture

### Required dependency

**tmux ≥ 3.0** becomes a hard runtime dependency for the TUI. (3.0 is required for `new-window -e` per-window env vars, used to inject `WORKTREE_NAME` and allocated ports.) If tmux is missing or too old, `wkspace` (no args) errors with an install hint and falls back to printing `--help`. The existing CLI subcommands are unaffected — they don't touch tmux.

### Two processes, one binary

- **`wkspace`** (no args) — the launcher. Detects/creates the tmux session, attaches, exits.
- **`wkspace --controller`** — a hidden subcommand. The ratatui app that runs inside the controller pane. Renders the sidebar, listens for keys, dispatches actions.

The existing CLI subcommands (`wkspace new <name>`, `wkspace list`, …) are unchanged.

### Session model

One tmux session per repo, named `wkspace-<repo-slug>`. The slug is derived from the repo's directory name with non-alphanumeric characters replaced by `-`, optionally suffixed with a short hash of the absolute path to disambiguate identically-named repos in different locations (deferred to v2 unless collisions surface).

Behaviour of `wkspace` (no args):

| Caller's tmux state | Behaviour |
|---|---|
| Not inside any tmux session | Create the session if needed, attach. |
| Inside the wkspace session for **this** repo | Refocus the controller pane (`tmux select-pane`) and exit. No new processes. |
| Inside **any other** tmux session | Error and exit 1. Print actionable instructions: detach with `Ctrl-B d` and re-run, or `tmux switch-client -t <session>` if the wkspace session for this repo already exists. |

Rationale for the third case: `tmux switch-client` would technically work (it swaps the client to a different session, no nesting), but yanking the user out of their current session without consent is surprising. Erroring keeps wkspace out of the user's existing tmux state.

### Controller pane

On session creation, the **home window** is split into:

- **Left pane** (~30 columns, resizable via tmux's normal pane-resize keys): runs `wkspace --controller`. This is the sidebar.
- **Right pane**: a regular shell (`$SHELL`) running in the repo root, with a brief one-time welcome banner ("Pick a worktree on the left, or press `n` to start one"). User can use it freely for quick repo-level commands.

When the user activates a worktree, a **new tmux window** is created (named after the worktree) running that worktree's shell. The home window (with the controller pane) stays put. Switching between worktrees = tmux's normal window-switching (`Ctrl-B n`/`p`/`<num>` or via the controller).

### Sources of truth

The controller doesn't keep its own state store. Everything is derived on each refresh:

| State | Source |
|---|---|
| Worktree list | `git worktree list --porcelain` |
| Dirty status, last commit, staleness | Git probes (same as today's `wkspace list`) |
| Branch description | `git config branch.<name>.description` |
| Active session windows | `tmux list-windows -t wkspace-<slug>` |
| Allocated ports per session | Per-window env vars (`tmux show-environment -t <session>:<window>`); not persisted |
| Config | `.wkspace.toml` |

**Refresh strategy:** poll every 2s on a tick, plus immediate refresh after any user action. All probes are cheap subprocess calls. If polling proves expensive in large repos, the interval can be backed off later.

## Layout

```
┌──────────────────────────────┐
│ wkspace · vibes-wkspace      │  title bar (1 line)
│ main · 12 worktrees          │  base branch, count
├──────────────────────────────┤
│                              │
│ ● feat-login         ●       │  worktree list
│   payments-rework    ●  !    │
│ ▸ ui-redesign                │  ▸ marks selection
│   docs-update        ! ·     │
│   experiment-foo        ·    │
│                              │
│                              │
├──────────────────────────────┤
│ feat-login                   │  detail block (~6 lines)
│ branch:  rob/feat-login      │
│ commit:  2 hours ago         │
│ status:  3 uncommitted       │
│ session: active (window 2)   │
│ desc:    Login form rework   │
├──────────────────────────────┤
│ n new   f from   o open      │  keymap footer (always visible)
│ d rm    s setup  t teardown  │
│ ?  help    q  quit/detach    │
├──────────────────────────────┤
│                              │  status line (transient messages)
└──────────────────────────────┘
```

### Row markers

Compact, right-aligned, semantic colour:

- **`●`** (green) — has an active tmux window
- **`!`** (yellow) — uncommitted changes
- **`·`** (dim) — stale (commit older than `stale_days`)

Rows whose path contains the user's external `cwd` are highlighted (inverted background).

### Selection & detail block

Selection is moved with `↑`/`↓` (or `j`/`k`). The detail block updates as selection moves and pulls from the same data the existing `wkspace list` uses, with an added "session" line (active / inactive) for whether a tmux window currently exists for that worktree.

### Empty state

If no worktrees exist, the list area shows a centred hint: "No worktrees yet. Press `n` to create one or `f` to start from an existing branch."

### Colour

Minimal and semantic. Respects the user's terminal theme. No custom colour scheme to maintain.

## Keybindings

### Navigation

| Key | Action |
|---|---|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `g` / `G` | Top / bottom of list |
| `enter` | Same as `o` (open / focus session) |
| `?` | Toggle help overlay |
| `r` | Manual refresh |
| `q` | Detach tmux session, exit controller |
| `Ctrl-C` | Same as `q` |
| `Q` (capital) | Kill the entire wkspace session for this repo, including all open windows. Confirmation prompt. |

### Actions on the selected worktree

| Key | Action |
|---|---|
| `o` / `enter` | If a session window exists → `tmux select-window`. Else → spawn a window in the worktree dir running `exec $SHELL`, switch to it. |
| `d` | Remove. If a window exists → kill it. If dirty → `[y/N]` confirmation. Then run teardown → `git worktree remove` → `git branch -D`. |
| `s` | Re-run setup. If a window exists → `tmux send-keys` the setup commands into it (visible in scrollback). Else → spawn a window with setup as startup. |
| `t` | Re-run teardown. Same window-handling as `s`. |

### Actions that don't need a selection

| Key | Action |
|---|---|
| `n` | New worktree. Inline prompt at bottom of sidebar (vim-style): `name: ▮`. Optional second prompt for description. Then runs the standard `new` flow: allocate ports, fetch, create branch, `git worktree add`, spawn window with setup script. |
| `f` | From existing branch. Fuzzy picker overlay fills the sidebar (search field at top, filtered list below). Lists local + remote branches, excluding the base branch and any already attached to a worktree. `enter` selects, `esc` cancels, then runs the standard `from` flow. |

### Modal styles

- **Single-line input** (name, description, dirty-rm confirmation): replaces the keymap footer temporarily, vim-style. Lightweight, doesn't disrupt the list view. `esc` cancels and restores the footer.
- **Picker overlay** (branch selection): fills the sidebar list area with a search field + filtered list. Title bar and detail block remain visible; keymap footer shows picker-specific bindings (`enter` select, `esc` cancel). Reuses the fuzzy logic that `dialoguer` provides today.

## Action flows

### `o` (open / focus)

1. Selected worktree's window already exists → `tmux select-window -t <session>:<name>`.
2. Otherwise:
   - `tmux new-window -t <session> -n <name> -c <worktree-dir> -e WORKTREE_NAME=<name> -e <port env vars> 'exec $SHELL'`
   - `tmux select-window -t <session>:<name>`
3. Controller refreshes; row's `●` marker now appears.

### `n` (new)

1. Inline name prompt (with current `prefix` config applied to suggested name); optional `desc` prompt.
2. In the controller process:
   - Allocate ports.
   - `git fetch <remote>`.
   - Create branch from base.
   - `git worktree add <dir> <branch>`.
   - Set branch description if provided.
   - Run `post-new` hook (errors logged in status line).
3. Hand off to tmux:
   - `tmux new-window -t <session> -n <name> -c <dir> -e <env-vars> 'sh -c "<setup-cmds>; exec $SHELL"'`
   - `tmux select-window`
4. User watches setup happen. On success, drops into shell. On failure, drops into shell anyway with the error visible — `wkspace setup` from inside the shell can retry.

### `f` (from)

1. Fuzzy picker over `git for-each-ref` results (local + remote refs, minus base, minus already-attached).
2. Selection → in controller: fetch, fast-forward local branch, `git worktree add` (no `-b`), allocate ports, run `post-from` hook.
3. Spawn tmux window same as `n`.

### `d` (rm)

1. If selected worktree is dirty → `Remove with uncommitted changes? [y/N]` inline prompt.
2. If a window exists → `tmux kill-window -t <session>:<name>`. (This SIGKILLs the running shell and any setup-in-progress; teardown still runs after.)
3. In controller: run teardown scripts (errors abort removal — same as today's CLI), `git worktree remove`, `git branch -D`, `post-rm` hook.
4. Refresh sidebar.

### `s` / `t` (setup / teardown)

1. If window exists for this worktree:
   - `tmux send-keys -t <session>:<name> '<each setup line>' Enter` for each script command. Output appears in the existing window's scrollback.
2. If no window → spawn one (like `o`) with the script as startup, then `exec $SHELL`.

### `q` vs `Q`

- **`q`**: `tmux detach-client`. Session and all worktree windows keep running. Re-running `wkspace` re-attaches with everything alive.
- **`Q`**: `[y/N]` confirmation, then `tmux kill-session -t wkspace-<slug>`. Tears down all windows. Useful for "I'm done with this repo today."

### Window close (user types `exit`)

tmux closes the window automatically. On next refresh (≤2s), the sidebar removes the `●` marker for that worktree. The worktree itself is **not** removed — that requires `d`.

If all worktree windows close, only window 0 (the home view with the controller pane) remains. The session stays alive because window 0's controller pane is still running.

## Lifecycle & edge cases

| Event | Behaviour |
|---|---|
| Controller crashes | Tmux pane dies. Session and worktree windows keep running. Re-running `wkspace` re-spawns the controller. (Auto-respawn via `tmux respawn-pane` is a v2 enhancement.) |
| First-run, no `.wkspace.toml` | TUI shows a setup screen ("No config — create one with defaults? [Y/n]"). On accept → run existing `init` flow. |
| External worktree edits (`git worktree add` from outside wkspace) | Show up on next refresh. No special handling. |
| Setup script killed mid-run by `d` | Window is killed, which terminates the running shell and any in-progress setup. Teardown still runs after. Probably fine; documented behaviour. |
| Two repos, same dir name | Currently produces colliding session slugs. Risk noted; mitigation (path-hash suffix) deferred. |
| User passes `--no-shell` style args via TUI | Not exposed in TUI for v1. CLI subcommands remain available for that. |

## Hooks

`post-new`, `post-from`, `post-rm`, `post-open`, `post-setup`, `post-teardown`, `post-list` — all continue to fire with the same env vars and rules they have today.

In TUI mode, hooks run in the controller process before window handoff. Hook stdout/stderr is captured and surfaced in the controller's status line (the line below the keymap footer; not shown above for brevity, but it exists for transient messages).

**Future improvement (v2):** route hook output into the session window via `tmux send-keys` so the user sees it inline alongside setup output.

## Errors & status messages

The sidebar reserves one line at the very bottom for transient status messages. Errors (red), warnings (yellow), info (default colour) live here.

- Git op fails (branch exists, fetch fails, conflicts) → red status line. Action aborted, sidebar state unchanged.
- tmux command fails (rare) → red status line.
- Setup script fails → drops into shell as designed; sidebar shows "setup-failed" mark on the row.
- Successful actions clear the line on next refresh.

## Phasing

Five milestones. Each ships something user-visible (except M3, which is a pure refactor). Stop and test/review at each gate before continuing.

### Milestone 1 — Skeleton (read-only)

**Scope:**
- Add `ratatui` + `crossterm` dependencies.
- Tmux preflight check.
- Hidden `wkspace --controller` subcommand.
- New no-args behaviour: tmux session detect/create, controller pane spawn, attach. Error path for inside-other-tmux.
- Controller renders full layout (title, list pulled from existing `git::list_worktrees`, detail block, keymap footer, status line).
- Navigation keys: `↑`/`↓`/`j`/`k`/`g`/`G`. Refresh: `r`. Help overlay: `?`. Quit/detach: `q`. Kill-all: `Q`.
- No mutating actions yet.

**Definition of done:**
- `wkspace` (no args) launches into the TUI in a fresh repo and an existing one with worktrees.
- All three tmux-state boot scenarios behave as specified.
- Layout renders correctly at multiple terminal sizes; selection and refresh work.
- `cargo test`/`clippy`/`fmt` clean.

**Test cadence:**
- Manual smoke: launch in/out of tmux, verify session naming, layout renders, nav keys, detach/re-attach, error message inside another session.
- Optional: snapshot test of layout via ratatui's test backend.

### Milestone 2 — Open & focus

**Scope:**
- `o` and `enter`: switch to existing window or spawn a new bare-shell window in the worktree dir with `WORKTREE_NAME` set.
- `●` marker on rows with active windows. Marker updates after window open/close (next refresh tick).

**Definition of done:**
- `o` on a worktree spawns the window correctly with env vars set (verified via `echo $WORKTREE_NAME`).
- Opening the same worktree twice reuses the existing window — long-running processes survive (test: leave `sleep 1000` running, switch away, switch back, verify still running).
- Marker appears/disappears as expected.

### Milestone 3 — Refactor command core

**Scope:**
- Pull pure logic out of `commands::new::run`, `commands::from::run`, `commands::rm::run`, `commands::setup::run`, `commands::teardown::run` into reusable functions that return data instead of printing or spawning shells.
- Existing CLI subcommands become thin wrappers around these functions, doing their own I/O (println, dialoguer, shell spawn). Behaviour identical to today.

**Definition of done:**
- Zero behavioural change to CLI subcommands. All existing integration tests in `tests/` pass untouched.
- New pure functions exist with at minimum unit-test coverage of happy paths.
- `cargo test`/`clippy`/`fmt` clean.

### Milestone 4 — Mutating actions

**Scope:**
- `n` — inline name + optional desc prompt → `new::create` → spawn window with setup as startup + `exec $SHELL`.
- `f` — branch picker overlay with fuzzy filter → `from::create` → spawn window.
- `d` — confirmation if dirty → kill window if any → `rm::remove`.
- `s`/`t` — `send-keys` to existing window or spawn fresh.
- Status markers: dirty `!`, stale `·`, current-cwd row highlight.

**Definition of done:**
- Full action matrix manually verified:
  - `n` happy path; `n` with conflicting name (graceful error in status line).
  - `f` happy path; `f` cancelled with `esc`.
  - `d` clean; `d` dirty with confirm; `d` dirty cancelled; `d` while window open (kills window cleanly).
  - `s`/`t` with and without existing window.
  - Setup failure drops into shell as designed.
- New integration tests for the refactored functions.

### Milestone 5 — Polish

**Scope:**
- `Q` confirmation prompt + `kill-session`.
- First-run config flow (no `.wkspace.toml` present).
- README rewrite: TUI section, tmux dependency note, screenshot or asciinema demo.
- Hook output routing decision: log to status line for v1, route into session window deferred to v2.

**Definition of done:**
- README accurately describes the TUI and the tmux requirement.
- All visual polish in place; no obvious rough edges.
- Final full CI run clean.

## Open questions / risks

These are noted for tracking but not blocking the design. Most are addressable post-v1.

1. **Repo slug collisions.** Two repos with identical directory names produce identical session slugs. Mitigation: append a short hash of the absolute path. Defer until reported.
2. **tmux ≥ 3.0 requirement.** Documented as a hard runtime requirement for the TUI. CLI subcommands unaffected. macOS Homebrew tmux is well past 3.0; some long-LTS Linux distros may ship older. Acceptable.
3. **TUI testing strategy.** ratatui has a test backend that lets us snapshot-render layouts. Action handlers can be tested with a mocked tmux interface. The existing integration tests cover the CLI subcommands and (after M3) the underlying logic.
4. **Hook output visibility in TUI mode.** v1: log to status line. v2: route into the session window via `send-keys` or a dedicated log pane. Acceptable trade-off for v1.
5. **`init` integration.** First-run prompt in TUI; existing `wkspace init` CLI subcommand still available. No conflict.

## Out of scope (for this design)

These were considered and explicitly deferred:

- Splits inside the tmux main pane (multiple sessions visible simultaneously).
- Custom themes / colour schemes.
- Per-worktree scratch notes or annotations.
- A plugin/extension system for custom keybindings or actions.
- Native Windows support beyond what tmux provides via WSL.
- Auto-respawn of a crashed controller pane.
