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
            Constraint::Length(2), // title bar
            Constraint::Min(3),    // worktree list
            Constraint::Length(7), // detail block
            Constraint::Length(3), // keymap footer
            Constraint::Length(1), // status line
        ])
        .split(area);

    render_title(f, chunks[0], app);
    render_list(f, chunks[1], app);
    render_detail(f, chunks[2], app);
    render_footer(f, chunks[3], app);
    render_status(f, chunks[4], app);

    if let crate::tui::controller::Modal::Help = app.modal {
        render_help_overlay(f, area);
    }
    if let crate::tui::controller::Modal::KillConfirm = app.modal {
        render_kill_confirm(f, area);
    }
}

fn render_title(f: &mut Frame, area: Rect, app: &App) {
    let line1 = format!("wkspace · {}", app.repo_name);
    let count = app.rows.len();
    let line2 = format!("{} · {} worktrees", app.base_branch, count);
    let p = Paragraph::new(vec![
        Line::from(Span::styled(
            line1,
            Style::default().add_modifier(Modifier::BOLD),
        )),
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

    let lines: Vec<Line> = app
        .rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
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
        })
        .collect();

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
        Line::from(Span::styled(
            &row.name,
            Style::default().add_modifier(Modifier::BOLD),
        )),
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
        Line::from(format!(
            "session: {}",
            if row.has_session { "active" } else { "—" }
        )),
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

fn render_help_overlay(f: &mut Frame, area: Rect) {
    let block = Block::default().title(" Help ").borders(Borders::ALL);
    let lines = vec![
        Line::from(""),
        Line::from("  ↑/↓ k/j      navigate"),
        Line::from("  g  G         top / bottom"),
        Line::from("  enter / o    open or focus session (M2)"),
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
    let block = Block::default()
        .title(" Kill session? ")
        .borders(Borders::ALL);
    let p = Paragraph::new(vec![
        Line::from(""),
        Line::from("  This will close all worktree windows in this repo's"),
        Line::from("  wkspace session. Long-running processes (Claude,"),
        Line::from("  dev servers) will be terminated."),
        Line::from(""),
        Line::from("  Press y to confirm, n or esc to cancel."),
    ])
    .block(block);
    let overlay = centered_rect(60, 8, area);
    f.render_widget(ratatui::widgets::Clear, overlay);
    f.render_widget(p, overlay);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}
