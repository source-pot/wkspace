use crate::tui::tmux::TmuxEnv;

pub struct Inputs {
    pub env: TmuxEnv,
    pub target_session: String,
    pub current_session: Option<String>,
    pub target_session_exists: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    CreateAndAttach,
    Attach,
    RefocusController,
    ErrorInsideOtherSession { target_exists: bool },
}

pub fn decide(inputs: Inputs) -> Decision {
    match inputs.env {
        TmuxEnv::Outside => {
            if inputs.target_session_exists {
                Decision::Attach
            } else {
                Decision::CreateAndAttach
            }
        }
        TmuxEnv::Inside => {
            if inputs.current_session.as_deref() == Some(inputs.target_session.as_str()) {
                Decision::RefocusController
            } else {
                Decision::ErrorInsideOtherSession {
                    target_exists: inputs.target_session_exists,
                }
            }
        }
    }
}
