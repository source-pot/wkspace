use ratatui::{backend::TestBackend, Terminal};
use wkspace::tui::controller::{data::WorktreeRow, view, App, Modal, Status};

fn empty_app() -> App {
    App {
        repo_root: "/tmp/foo".into(),
        repo_name: "foo".into(),
        base_branch: "main".into(),
        rows: vec![],
        selected: 0,
        modal: Modal::None,
        status: Status::default(),
        should_quit: false,
        kill_session: false,
    }
}

fn one_row(name: &str) -> WorktreeRow {
    WorktreeRow {
        name: name.into(),
        branch: name.into(),
        uncommitted: 0,
        last_commit: "1 hour ago".into(),
        stale: false,
        description: "".into(),
        has_session: false,
    }
}

#[test]
fn renders_title_bar_with_repo_name_and_count() {
    let backend = TestBackend::new(30, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = empty_app();
    terminal.draw(|f| view::render(f, &app)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let line0: String = (0..30)
        .map(|x| buffer[(x, 0)].symbol().to_string())
        .collect();
    assert!(line0.contains("wkspace"));
    assert!(line0.contains("foo"));
}

#[test]
fn empty_state_shows_hint() {
    let backend = TestBackend::new(40, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = empty_app();
    terminal.draw(|f| view::render(f, &app)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let mut dump = String::new();
    for y in 0..20 {
        for x in 0..40 {
            dump.push_str(buffer[(x, y)].symbol());
        }
    }
    assert!(dump.contains("No worktrees"));
}

#[test]
fn renders_row_names() {
    let backend = TestBackend::new(40, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = empty_app();
    app.rows = vec![one_row("alpha"), one_row("beta")];
    terminal.draw(|f| view::render(f, &app)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let mut dump = String::new();
    for y in 0..20 {
        for x in 0..40 {
            dump.push_str(buffer[(x, y)].symbol());
        }
    }
    assert!(dump.contains("alpha"));
    assert!(dump.contains("beta"));
}
