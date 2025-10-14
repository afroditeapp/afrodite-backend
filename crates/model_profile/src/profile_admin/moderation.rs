use diesel::prelude::*;
use model_server_data::ProfileStringModerationContentType;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    AccountId, ProfileStringModerationInfo, ProfileStringModerationRejectedReasonCategory,
    ProfileStringModerationRejectedReasonDetails,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProfileStringPendingModerationList {
    pub values: Vec<ProfileStringPendingModeration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams)]
pub struct GetProfileStringPendingModerationParams {
    pub content_type: ProfileStringModerationContentType,
    pub show_values_which_bots_can_moderate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct ProfileStringPendingModeration {
    pub id: AccountId,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct PostModerateProfileString {
    pub id: AccountId,
    pub value: String,
    pub content_type: ProfileStringModerationContentType,
    pub accept: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_category: Option<ProfileStringModerationRejectedReasonCategory>,
    pub rejected_details: ProfileStringModerationRejectedReasonDetails,
    /// If true, ignore accept, rejected_category, rejected_details and move
    /// the text to waiting for human moderation state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_to_human: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams)]
pub struct GetProfileStringStateParams {
    pub content_type: ProfileStringModerationContentType,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProfileStringState {
    /// If empty, the `moderation_info` is `None`.
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation_info: Option<ProfileStringModerationInfo>,
}
