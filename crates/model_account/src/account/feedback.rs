use serde::{Deserialize, Deserializer, Serialize, de::Error};
use simple_backend_model::NonEmptyString;
use utoipa::ToSchema;

/// Send feedback to the server admins.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct SendFeedback {
    /// Feedback category key from [model::FeedbackCategory::key].
    pub category_key: String,
    /// Feedback body string. The string must be non-empty after trimming.
    #[serde(deserialize_with = "deserialize_trimmed_non_empty_string")]
    pub body: NonEmptyString,
}

fn deserialize_trimmed_non_empty_string<'de, D>(deserializer: D) -> Result<NonEmptyString, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    NonEmptyString::from_string(value.trim().to_string())
        .ok_or_else(|| D::Error::custom("NonEmptyString cannot be empty"))
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, ToSchema)]
pub struct SendFeedbackResult {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error: bool,
    /// The category key is not in the current server config. The client
    /// should fetch the feedback config again.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_unknown_category: bool,
    /// The feedback body is longer than the configured limit.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_body_too_long: bool,
    /// Email sending failed.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_email_sending_failed: bool,
}

impl SendFeedbackResult {
    pub fn success() -> Self {
        Default::default()
    }

    pub fn unknown_category() -> Self {
        Self {
            error: true,
            error_unknown_category: true,
            ..Default::default()
        }
    }

    pub fn body_too_long() -> Self {
        Self {
            error: true,
            error_body_too_long: true,
            ..Default::default()
        }
    }

    pub fn email_sending_failed() -> Self {
        Self {
            error: true,
            error_email_sending_failed: true,
            ..Default::default()
        }
    }
}
