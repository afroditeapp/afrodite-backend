use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostAccountVerificationQueueItem {
    pub verification_method: String,
    pub verification_data: String,
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct AccountVerificationQueueStatus {
    /// The first queue position is 1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_position: Option<u32>,
}
