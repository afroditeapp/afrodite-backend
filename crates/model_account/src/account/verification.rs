use diesel::Queryable;
pub use model::AccountVerificationQueueItem as PostAccountVerificationQueueItem;
use model::{AccountVerificationErrorFlagsValue, UnixTime, VerificationMethod};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Queryable)]
pub struct AccountVerificationDataInternal {
    pub verification_method: Option<VerificationMethod>,
    pub verification_unix_time: Option<UnixTime>,
    pub verification_error_flags: Option<AccountVerificationErrorFlagsValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PostAccountVerificationQueueItemResult {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_already_in_queue: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_queue_full: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_initial_setup_not_completed: bool,
}

impl PostAccountVerificationQueueItemResult {
    pub fn success() -> Self {
        Self::default()
    }

    pub fn error_already_in_queue() -> Self {
        Self {
            error: true,
            error_already_in_queue: true,
            ..Default::default()
        }
    }

    pub fn error_queue_full() -> Self {
        Self {
            error: true,
            error_queue_full: true,
            ..Default::default()
        }
    }

    pub fn error_initial_setup_not_completed() -> Self {
        Self {
            error: true,
            error_initial_setup_not_completed: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct AccountVerificationQueueStatus {
    /// The first queue position is 1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_position: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_method: Option<VerificationMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_unix_time: Option<UnixTime>,
    /// Null means there are no known verification errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_error_flags: Option<AccountVerificationErrorFlagsValue>,
}
