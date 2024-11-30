use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    AccountId, ProfileTextModerationRejectedReasonCategory,
    ProfileTextModerationRejectedReasonDetails,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProfileTextPendingModerationList {
    pub values: Vec<ProfileTextPendingModeration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams)]
pub struct GetProfileTextPendingModerationParams {
    pub show_texts_which_bots_can_moderate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct ProfileTextPendingModeration {
    pub id: AccountId,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct PostModerateProfileText {
    pub id: AccountId,
    pub text: String,
    pub accept: bool,
    pub rejected_category: Option<ProfileTextModerationRejectedReasonCategory>,
    pub rejected_details: Option<ProfileTextModerationRejectedReasonDetails>,
    /// If true, ignore accept, rejected_category, rejected_details and move
    /// the text to waiting for human moderation state.
    pub move_to_human: Option<bool>,
}
