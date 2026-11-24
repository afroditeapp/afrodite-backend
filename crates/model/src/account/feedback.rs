use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::StringResource;

/// Feedback form config which the client fetches when the feedback
/// screen is opened.
#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeedbackConfig {
    /// Max feedback body length in grapheme clusters.
    pub body_max_grapheme_cluster_count: u16,
    /// Categories which the client should show. Categories which are
    /// hidden in the server config are not included.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(default = json!([]))]
    pub categories: Vec<FeedbackCategory>,
}

/// Feedback category.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeedbackCategory {
    /// Stable category key which the client sends back when sending
    /// feedback. The key is validated against the current server config
    /// on every send.
    pub key: String,
    /// Category name.
    pub name: StringResource,
}
