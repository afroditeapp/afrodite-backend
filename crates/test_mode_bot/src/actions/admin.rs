use super::{BotAction, BotState};

pub mod content;
pub mod profile_text;

struct EmptyPage;

/// Default state is reject.
#[derive(Default)]
pub struct ModerationResult {
    pub accept: bool,
    pub move_to_human: bool,
    pub rejected_details: Option<String>,
    pub delete: bool,
}

impl ModerationResult {
    pub fn error() -> Self {
        Self {
            rejected_details: Some(
                "Error occurred. Try again and if this continues, please contact customer support."
                    .to_string(),
            ),
            ..Default::default()
        }
    }

    pub fn reject(details: Option<&str>) -> Self {
        Self {
            rejected_details: details.map(|text| text.to_string()),
            ..Default::default()
        }
    }

    pub fn accept() -> Self {
        Self {
            accept: true,
            ..Default::default()
        }
    }

    pub fn move_to_human() -> Self {
        Self {
            move_to_human: true,
            ..Default::default()
        }
    }

    pub fn delete() -> Self {
        Self {
            delete: true,
            ..Default::default()
        }
    }

    pub fn is_deleted_or_rejected(&self) -> bool {
        !self.accept && !self.move_to_human
    }

    pub fn is_move_to_human(&self) -> bool {
        self.move_to_human
    }
}

enum LlmModerationResult {
    StopModerationSesssion,
    Decision(Option<ModerationResult>),
}
