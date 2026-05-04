use crate::tui::controller::{App, Modal};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Refresh,
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    if matches!(app.modal, Modal::Help) {
        return handle_help(app, key);
    }
    if matches!(app.modal, Modal::KillConfirm) {
        return handle_kill_confirm(app, key);
    }

    match (key.code, key.modifiers) {
        (KeyCode::Char('j') | KeyCode::Down, _) => move_down(app),
        (KeyCode::Char('k') | KeyCode::Up, _) => move_up(app),
        (KeyCode::Char('g'), m) if !m.contains(KeyModifiers::SHIFT) => app.selected = 0,
        (KeyCode::Char('G'), _) => {
            if !app.rows.is_empty() {
                app.selected = app.rows.len() - 1;
            }
        }
        (KeyCode::Char('q'), _) => app.should_quit = true,
        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => app.should_quit = true,
        (KeyCode::Char('Q'), _) => app.modal = Modal::KillConfirm,
        (KeyCode::Char('?'), _) => app.modal = Modal::Help,
        (KeyCode::Char('r'), _) => return Action::Refresh,
        _ => {}
    }
    Action::None
}

fn move_down(app: &mut App) {
    if !app.rows.is_empty() && app.selected + 1 < app.rows.len() {
        app.selected += 1;
    }
}

fn move_up(app: &mut App) {
    if app.selected > 0 {
        app.selected -= 1;
    }
}

fn handle_help(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('?') | KeyCode::Esc => app.modal = Modal::None,
        _ => {}
    }
    Action::None
}

fn handle_kill_confirm(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.kill_session = true;
            app.should_quit = true;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.modal = Modal::None;
        }
        _ => {}
    }
    Action::None
}
