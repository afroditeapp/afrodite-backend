use diesel::prelude::*;
use model::{AccountId, ContentId};
use model_server_data::MediaContentType;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    ProfileContentModerationRejectedReasonCategory, ProfileContentModerationRejectedReasonDetails,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
pub enum ModerationQueueType {
    MediaModeration,
    InitialMediaModeration,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetProfileContentPendingModerationParams {
    pub content_type: MediaContentType,
    pub queue: ModerationQueueType,
    pub show_content_which_bots_can_moderate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProfileContentPendingModerationList {
    pub values: Vec<ProfileContentPendingModeration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct ProfileContentPendingModeration {
    pub account_id: AccountId,
    pub content_id: ContentId,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct PostModerateProfileContent {
    pub account_id: AccountId,
    pub content_id: ContentId,
    pub accept: bool,
    pub rejected_category: Option<ProfileContentModerationRejectedReasonCategory>,
    pub rejected_details: Option<ProfileContentModerationRejectedReasonDetails>,
    /// If true, ignore accept, rejected_category, rejected_details and move
    /// the content to waiting for human moderation state.
    pub move_to_human: Option<bool>,
}
