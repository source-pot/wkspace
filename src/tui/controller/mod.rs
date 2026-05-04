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
        let repo_name = ctx
            .repo_root
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
    // Real run loop comes in Task 14.
    let _ = &mut app;
    println!("controller boot ok ({} rows)", app.rows.len());
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    Ok(())
}
