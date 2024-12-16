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
