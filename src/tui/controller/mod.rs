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
    pub fn new(ctx: &context::Context) -> Self {
        let (rows, status) = match data::fetch_rows(ctx) {
            Ok(r) => (r, Status::default()),
            Err(e) => (
                Vec::new(),
                Status {
                    message: Some(format!("initial fetch failed: {e}")),
                    level: StatusLevel::Error,
                },
            ),
        };
        let repo_name = ctx
            .repo_root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        Self {
            repo_root: ctx.repo_root.clone(),
            repo_name,
            base_branch: ctx.config.worktree.base_branch.clone(),
            rows,
            selected: 0,
            modal: Modal::None,
            status,
            should_quit: false,
            kill_session: false,
        }
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
    use crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::io::stdout;
    use std::time::{Duration, Instant};

    let cwd = env::current_dir().context("get cwd")?;
    let ctx = context::resolve(&cwd)?;
    let mut app = App::new(&ctx);

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
        let _ = std::process::Command::new("tmux")
            .arg("detach-client")
            .status();
    }

    result
}
