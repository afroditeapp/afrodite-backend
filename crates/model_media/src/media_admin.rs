use diesel::prelude::*;
use model::{AccountId, ContentId};
use model_server_data::MediaContentType;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    MediaContentModerationRejectedReasonCategory, MediaContentModerationRejectedReasonDetails,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
pub enum ModerationQueueType {
    MediaModeration,
    InitialMediaModeration,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetMediaContentPendingModerationParams {
    pub content_type: MediaContentType,
    pub queue: ModerationQueueType,
    pub show_content_which_bots_can_moderate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetMediaContentPendingModerationList {
    pub values: Vec<MediaContentPendingModeration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct MediaContentPendingModeration {
    pub account_id: AccountId,
    pub content_id: ContentId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_category: Option<MediaContentModerationRejectedReasonCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_details: Option<MediaContentModerationRejectedReasonDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct PostModerateMediaContent {
    pub account_id: AccountId,
    pub content_id: ContentId,
    pub accept: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_category: Option<MediaContentModerationRejectedReasonCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_details: Option<MediaContentModerationRejectedReasonDetails>,
    /// If true, ignore accept and move the content to waiting for human moderation state.
    /// rejected_category and rejected_details can be used to set the reason why the bot
    /// moved the content to human moderation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_to_human: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct PostMediaContentFaceDetectedValue {
    pub account_id: AccountId,
    pub content_id: ContentId,
    /// Set to None to clear the manual override and use the automatic detection value
    pub value: Option<bool>,
}
