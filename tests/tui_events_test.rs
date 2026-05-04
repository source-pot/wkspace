use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use wkspace::tui::controller::{
    data::WorktreeRow,
    events::{handle_key, Action},
    App, Modal, Status,
};

fn app_with_rows(n: usize) -> App {
    App {
        repo_root: "/tmp/foo".into(),
        repo_name: "foo".into(),
        base_branch: "main".into(),
        rows: (0..n)
            .map(|i| WorktreeRow {
                name: format!("w{i}"),
                branch: format!("w{i}"),
                uncommitted: 0,
                last_commit: "—".into(),
                stale: false,
                description: "".into(),
                has_session: false,
            })
            .collect(),
        selected: 0,
        modal: Modal::None,
        status: Status::default(),
        should_quit: false,
        kill_session: false,
    }
}

fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

#[test]
fn down_increments_selection() {
    let mut app = app_with_rows(3);
    handle_key(&mut app, key('j'));
    assert_eq!(app.selected, 1);
}

#[test]
fn up_at_top_clamps() {
    let mut app = app_with_rows(3);
    handle_key(&mut app, key('k'));
    assert_eq!(app.selected, 0);
}

#[test]
fn down_at_bottom_clamps() {
    let mut app = app_with_rows(2);
    app.selected = 1;
    handle_key(&mut app, key('j'));
    assert_eq!(app.selected, 1);
}

#[test]
fn g_jumps_to_top() {
    let mut app = app_with_rows(5);
    app.selected = 4;
    handle_key(&mut app, key('g'));
    assert_eq!(app.selected, 0);
}

#[test]
fn capital_g_jumps_to_bottom() {
    let mut app = app_with_rows(5);
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT),
    );
    assert_eq!(app.selected, 4);
}

#[test]
fn q_sets_should_quit() {
    let mut app = app_with_rows(1);
    handle_key(&mut app, key('q'));
    assert!(app.should_quit);
}

#[test]
fn ctrl_c_quits() {
    let mut app = app_with_rows(1);
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
    );
    assert!(app.should_quit);
}

#[test]
fn question_mark_toggles_help() {
    let mut app = app_with_rows(1);
    handle_key(&mut app, key('?'));
    assert!(matches!(app.modal, Modal::Help));
    handle_key(&mut app, key('?'));
    assert!(matches!(app.modal, Modal::None));
}

#[test]
fn esc_dismisses_help() {
    let mut app = app_with_rows(1);
    app.modal = Modal::Help;
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(matches!(app.modal, Modal::None));
}

#[test]
fn capital_q_opens_confirm() {
    let mut app = app_with_rows(1);
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::SHIFT),
    );
    assert!(matches!(app.modal, Modal::KillConfirm));
}

#[test]
fn y_in_kill_confirm_sets_kill_flag() {
    let mut app = app_with_rows(1);
    app.modal = Modal::KillConfirm;
    handle_key(&mut app, key('y'));
    assert!(app.kill_session);
    assert!(app.should_quit);
}

#[test]
fn n_in_kill_confirm_cancels() {
    let mut app = app_with_rows(1);
    app.modal = Modal::KillConfirm;
    handle_key(&mut app, key('n'));
    assert!(!app.kill_session);
    assert!(matches!(app.modal, Modal::None));
}

#[test]
fn refresh_action_returned() {
    let mut app = app_with_rows(1);
    let action = handle_key(&mut app, key('r'));
    assert_eq!(action, Action::Refresh);
}

#[test]
fn other_keys_do_nothing() {
    let mut app = app_with_rows(1);
    let before = app.selected;
    handle_key(&mut app, key('z'));
    assert_eq!(app.selected, before);
    assert!(!app.should_quit);
}
