use std::{fmt::Debug, time::Instant};

use profile_text::ProfileTextModerationState;

use super::{BotAction, BotState};

pub mod profile_text;
pub mod content;

struct EmptyPage;

#[derive(Debug, Default)]
pub struct AdminBotState {
    profile_content_moderation_started: Option<Instant>,
    profile_text: Option<ProfileTextModerationState>,
}
