use super::{BotAction, BotState};

pub mod content;
pub mod profile_string;

struct EmptyPage;

/// Default state is reject.
#[derive(Default, Clone)]
pub struct ModerationResult {
    pub accept: bool,
    pub move_to_human: bool,
    pub rejected_details: Option<String>,
    pub delete: bool,
}

impl ModerationResult {
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

    pub fn move_to_human(details: Option<String>) -> Self {
        Self {
            move_to_human: true,
            rejected_details: details,
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
