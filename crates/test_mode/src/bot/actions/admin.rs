use content::ContentModerationState;
use profile_text::ProfileTextModerationState;

use super::{BotAction, BotState};

pub mod profile_text;
pub mod content;

struct EmptyPage;

#[derive(Debug, Default)]
pub struct AdminBotState {
    profile_text: Option<ProfileTextModerationState>,
    content: Option<ContentModerationState>,
}

pub struct ModerationResult {
    pub accept: bool,
    pub move_to_human: bool,
    pub rejected_details: Option<String>,
}

impl ModerationResult {
    pub fn error() -> Self {
        Self {
            accept: false,
            move_to_human: false,
            rejected_details: Some("Error occurred. Try again and if this continues, please contact customer support.".to_string()),
        }
    }

    pub fn reject(details: Option<&str>) -> Self {
        Self {
            accept: false,
            move_to_human: false,
            rejected_details: details.map(|text| text.to_string()),
        }
    }

    pub fn accept() -> Self {
        Self {
            accept: true,
            move_to_human: false,
            rejected_details: None,
        }
    }

    pub fn move_to_human() -> Self {
        Self {
            accept: false,
            move_to_human: true,
            rejected_details: None,
        }
    }
}
