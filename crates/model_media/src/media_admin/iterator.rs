use model::{AccountId, AccountIdDb, ContentId, ContentIdDb, ProfileAge, UnixTime};
use serde::{Deserialize, Serialize};
use simple_backend_model::NonEmptyString;
use utoipa::ToSchema;

use crate::{
    ContentModerationState, MediaContentModerationRejectedReasonCategory,
    MediaContentModerationRejectedReasonDetails,
};

/// Media content ID DB value.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MediaContentIdDbValue {
    #[schema(value_type = i64)]
    pub content_id_db: ContentIdDb,
}

/// Media content list filters.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct MediaContentFilters {
    /// Includes content in any of the provided moderation states.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub moderation_states: Option<Vec<ContentModerationState>>,
    /// Includes content which was uploaded when profile visibility was
    /// pending public or pending private.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub initial_content: Option<bool>,
    /// Includes content which the client captured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub secure_capture: Option<bool>,
    /// Includes content which is currently in use (usage start time is set
    /// and usage end time is not set) when true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub in_use: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub uploaded_after_unix_time: Option<UnixTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub uploaded_before_unix_time: Option<UnixTime>,
    /// Effective face detected value. Manual value overrides the automatic
    /// value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub face_detected: Option<bool>,
    /// Includes content where the manual face detected value is set when
    /// true and content where it is not set when false.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub face_detected_manual_set: Option<bool>,
    /// Includes content moderated by the provided account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub moderator_account_id: Option<AccountId>,
    /// Includes content which is not moderated by the provided account.
    /// Content without a moderator is included.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub moderator_account_id_not: Option<AccountId>,
}

/// Media content list extra info settings.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, ToSchema)]
pub struct MediaContentExtraInfoSettings {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub name: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub age: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub upload_time: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub secure_capture: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub face_detected: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub initial_content: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderator_account_id: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderation_state: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub rejected_reason_category: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub rejected_reason_details: bool,
}

/// Params for admin media content iterator page.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MediaContentIteratorSettings {
    #[schema(value_type = i64)]
    pub start_position: ContentIdDb,
    pub page: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub filters: Option<MediaContentFilters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub extra_info: Option<MediaContentExtraInfoSettings>,
}

impl MediaContentIteratorSettings {
    pub fn filters(&self) -> MediaContentFilters {
        self.filters.clone().unwrap_or_default()
    }

    pub fn extra_info(&self) -> MediaContentExtraInfoSettings {
        self.extra_info.unwrap_or_default()
    }
}

/// Media content list filters with account IDs resolved to database IDs.
/// Internal type.
#[derive(Debug, Clone, Default)]
pub struct MediaContentFiltersInternal {
    pub moderation_states: Option<Vec<ContentModerationState>>,
    pub initial_content: Option<bool>,
    pub secure_capture: Option<bool>,
    pub in_use: Option<bool>,
    pub uploaded_after_unix_time: Option<UnixTime>,
    pub uploaded_before_unix_time: Option<UnixTime>,
    pub face_detected: Option<bool>,
    pub face_detected_manual_set: Option<bool>,
    pub moderator_account_id: Option<AccountIdDb>,
    pub moderator_account_id_not: Option<AccountIdDb>,
}

/// Params for admin media content iterator page with account IDs resolved
/// to database IDs. Internal type.
#[derive(Debug, Clone)]
pub struct MediaContentIteratorSettingsInternal {
    pub start_position: ContentIdDb,
    pub page: i64,
    pub filters: MediaContentFiltersInternal,
    pub extra_info: MediaContentExtraInfoSettings,
}

/// Database data for one media content iterator page value. Internal type.
#[derive(Debug, Clone)]
pub struct MediaContentIteratorPageData {
    pub account_id: AccountId,
    pub content_id: ContentId,
    pub name: Option<NonEmptyString>,
    pub age: ProfileAge,
    pub upload_time: UnixTime,
    pub secure_capture: bool,
    /// Effective face detected value.
    pub face_detected: bool,
    pub initial_content: bool,
    pub moderation_state: ContentModerationState,
    pub rejected_reason_category: Option<MediaContentModerationRejectedReasonCategory>,
    pub rejected_reason_details: Option<MediaContentModerationRejectedReasonDetails>,
    pub moderator_account_id: Option<AccountId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MediaContentIteratorPageValue {
    pub account_id: AccountId,
    pub content_id: ContentId,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub name: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    #[schema(value_type = Option<i64>)]
    pub age: Option<ProfileAge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub upload_time: Option<UnixTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub secure_capture: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub face_detected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub initial_content: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub moderator_account_id: Option<AccountId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub moderation_state: Option<ContentModerationState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub rejected_reason_category: Option<MediaContentModerationRejectedReasonCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub rejected_reason_details: Option<MediaContentModerationRejectedReasonDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MediaContentIteratorPage {
    pub values: Vec<MediaContentIteratorPageValue>,
    /// Page number of the next page. None if the end of the iteration
    /// has been reached.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub next_page: Option<i64>,
}
