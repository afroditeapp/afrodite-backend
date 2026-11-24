use diesel::prelude::*;
use model::{AccountId, ContentId, ContentSlot, UnixTime};
use model_server_data::MediaContentType;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    ContentModerationState, MediaContentModerationRejectedReasonCategory,
    MediaContentModerationRejectedReasonDetails, MediaContentRaw,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
pub enum MediaContentModerationType {
    Normal,
    Initial,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
pub enum MediaContentModerationQueueType {
    WaitingAdminBot,
    WaitingAdmin,
    AcceptedByAdminBot,
    RejectedByAdminBot,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetMediaContentModerationQueuePageParams {
    pub content_type: MediaContentType,
    pub moderation_type: MediaContentModerationType,
    pub queue_type: MediaContentModerationQueueType,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MediaContentModerationQueuePage {
    pub values: Vec<MediaContentPendingModeration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetMediaContentFaceVerifiedNullList {
    pub values: Vec<MediaContentFaceVerifiedNullByAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MediaContentFaceVerifiedNullByAccount {
    pub account_id: AccountId,
    pub security_content: ContentId,
    pub values: Vec<ContentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct MediaContentPendingModeration {
    pub account_id: AccountId,
    pub content_id: ContentId,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub rejected_category: Option<MediaContentModerationRejectedReasonCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub rejected_details: Option<MediaContentModerationRejectedReasonDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct PostModerateMediaContent {
    pub account_id: AccountId,
    pub content_id: ContentId,
    pub accept: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub rejected_category: Option<MediaContentModerationRejectedReasonCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub rejected_details: Option<MediaContentModerationRejectedReasonDetails>,
    /// If true, ignore accept and move the content to waiting for human moderation state.
    /// rejected_category and rejected_details can be used to set the reason why the bot
    /// moved the content to human moderation.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub move_to_human: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct PostMediaContentFaceDetectedValue {
    pub account_id: AccountId,
    pub content_id: ContentId,
    /// Set to None to clear the manual override and use the automatic detection value
    pub value: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct PostMediaContentFaceVerifiedValue {
    pub account_id: AccountId,
    pub security_content: ContentId,
    pub values: Vec<PostMediaContentFaceVerifiedValueItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct PostMediaContentFaceVerifiedValueItem {
    pub content_id: ContentId,
    /// Bot sets automatic face verification value.
    /// Human admin sets manual override value.
    /// Set to None to clear the currently applicable value.
    pub value: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct PostSecurityContentVerifiedValue {
    pub account_id: AccountId,
    pub security_content: ContentId,
    /// Bot sets automatic security content verification value.
    /// Human admin sets manual override value.
    /// Set to None to clear the currently applicable value.
    pub value: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ContentInfoDetailedAdmin {
    pub cid: ContentId,
    pub ctype: MediaContentType,
    pub state: ContentModerationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub slot: Option<ContentSlot>,
    pub secure_capture: bool,
    /// Face detected (automatic)
    pub face_detected: bool,
    /// Manual face detected value set by admin
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub face_detected_manual: Option<bool>,
    /// Face verified against current security content (automatic or manual)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub face_verified: Option<bool>,
    /// Manual face verified value set by admin
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub face_verified_manual: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub usage_start_time: Option<UnixTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub usage_end_time: Option<UnixTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub rejected_reason_category: Option<MediaContentModerationRejectedReasonCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub rejected_reason_details: Option<MediaContentModerationRejectedReasonDetails>,
    /// Account which moderated the content
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub moderator_account_id: Option<AccountId>,
}

impl ContentInfoDetailedAdmin {
    pub fn new(value: MediaContentRaw, moderator_account_id: Option<AccountId>) -> Self {
        ContentInfoDetailedAdmin {
            cid: value.uuid,
            ctype: value.content_type_number,
            state: value.moderation_state,
            slot: value.slot_number(),
            secure_capture: value.secure_capture,
            face_detected: value.face_detected(),
            face_detected_manual: value.manual_face_detected(),
            face_verified: value.face_verified(),
            face_verified_manual: value.manual_face_verified(),
            usage_end_time: value.usage_end_unix_time,
            usage_start_time: value.usage_start_unix_time,
            rejected_reason_category: value.moderation_rejected_reason_category,
            rejected_reason_details: value.moderation_rejected_reason_details,
            moderator_account_id,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct AccountContentAdmin {
    pub data: Vec<ContentInfoDetailedAdmin>,
}
