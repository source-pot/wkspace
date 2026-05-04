use wkspace::tui::launcher::{decide, Decision, Inputs};
use wkspace::tui::tmux::TmuxEnv;

#[test]
fn outside_tmux_no_session_creates() {
    let d = decide(Inputs {
        env: TmuxEnv::Outside,
        is_target: false,
        target_session_exists: false,
    });
    assert_eq!(d, Decision::CreateAndAttach);
}

#[test]
fn outside_tmux_session_exists_attaches() {
    let d = decide(Inputs {
        env: TmuxEnv::Outside,
        is_target: false,
        target_session_exists: true,
    });
    assert_eq!(d, Decision::Attach);
}

#[test]
fn inside_target_session_refocuses() {
    let d = decide(Inputs {
        env: TmuxEnv::Inside,
        is_target: true,
        target_session_exists: true,
    });
    assert_eq!(d, Decision::RefocusController);
}

#[test]
fn inside_other_session_errors() {
    let d = decide(Inputs {
        env: TmuxEnv::Inside,
        is_target: false,
        target_session_exists: false,
    });
    assert!(matches!(
        d,
        Decision::ErrorInsideOtherSession {
            target_exists: false
        }
    ));
}

#[test]
fn inside_other_session_target_exists_errors_with_hint() {
    let d = decide(Inputs {
        env: TmuxEnv::Inside,
        is_target: false,
        target_session_exists: true,
    });
    assert!(matches!(
        d,
        Decision::ErrorInsideOtherSession {
            target_exists: true
        }
    ));
}
