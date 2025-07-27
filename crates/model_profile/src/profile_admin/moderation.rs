use diesel::prelude::*;
use model_server_data::ProfileStringModerationContentType;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    AccountId, ProfileModerationInfo, ProfileModerationRejectedReasonCategory,
    ProfileModerationRejectedReasonDetails,
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
    pub rejected_category: Option<ProfileModerationRejectedReasonCategory>,
    pub rejected_details: ProfileModerationRejectedReasonDetails,
    /// If true, ignore accept, rejected_category, rejected_details and move
    /// the text to waiting for human moderation state.
    pub move_to_human: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams)]
pub struct GetProfileStringModerationStateParams {
    pub content_type: ProfileStringModerationContentType,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProfileStringModerationState {
    /// If empty, the `moderation_info` is `None`.
    pub value: String,
    pub moderation_info: Option<ProfileModerationInfo>,
}
