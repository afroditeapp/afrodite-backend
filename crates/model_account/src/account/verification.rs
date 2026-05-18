use diesel::Queryable;
pub use model::AccountVerificationQueueItem as PostAccountVerificationQueueItem;
use model::{
    AccountVerificationErrorFlagsValue, AgeVerificationMethod, UnixTime, VerificationMethod,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Queryable)]
pub struct AccountVerificationDataInternal {
    pub verification_method: Option<VerificationMethod>,
    pub verification_unix_time: Option<UnixTime>,
    pub verification_error_flags: AccountVerificationErrorFlagsValue,
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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PostAgeVerification {
    pub verification_method: AgeVerificationMethod,
    pub verification_data: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct PostAgeVerificationResult {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_verification_method_not_configured: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_verification_data_parsing_failed: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_verification_data_verification_failed: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_age_under_18: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_age_already_verified: bool,
}

impl PostAgeVerificationResult {
    pub fn success() -> Self {
        Self::default()
    }

    pub fn error_verification_method_not_configured() -> Self {
        Self {
            error: true,
            error_verification_method_not_configured: true,
            ..Default::default()
        }
    }

    pub fn error_verification_data_parsing_failed() -> Self {
        Self {
            error: true,
            error_verification_data_parsing_failed: true,
            ..Default::default()
        }
    }

    pub fn error_verification_data_verification_failed() -> Self {
        Self {
            error: true,
            error_verification_data_verification_failed: true,
            ..Default::default()
        }
    }

    pub fn error_age_under_18() -> Self {
        Self {
            error: true,
            error_age_under_18: true,
            ..Default::default()
        }
    }

    pub fn error_age_already_verified() -> Self {
        Self {
            error: true,
            error_age_already_verified: true,
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
    /// Empty flags value means there are no known verification errors.
    pub verification_error_flags: AccountVerificationErrorFlagsValue,
}
