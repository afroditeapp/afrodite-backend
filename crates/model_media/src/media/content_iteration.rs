use serde::{Deserialize, Serialize};
use simple_backend_model::UnixTime;
use utoipa::{IntoParams, ToSchema};

use crate::{AccountId, ContentId};

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct GetMediaContentIterationPageParams {
    /// `true` returns only security content items. `false` or unset
    /// returns all content items (profile and security content).
    pub only_security_content: Option<bool>,
    /// Zero-based page number.
    pub page: i64,
    /// Iteration origin point. Max upload time value which can be returned.
    /// Descending iteration starts from this time. Values are always
    /// returned in descending upload time order.
    pub origin_unix_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MediaContentIterationValue {
    pub account_id: AccountId,
    pub content_id: ContentId,
    /// Face detected (automatic or manual)
    pub face_detected: bool,
    pub secure_capture: bool,
    pub uploaded_time: UnixTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MediaContentIterationPage {
    pub values: Vec<MediaContentIterationValue>,
}
