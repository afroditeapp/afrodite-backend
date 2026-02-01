use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const MAX_TOKENS_DEFAULT: u32 = 10_000;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, Default)]
pub enum ModerationAction {
    Accept,
    Reject,
    #[default]
    MoveToHuman,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct AdminBotConfig {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profile_name_moderation_enabled: bool,
    pub profile_name_moderation: AdminProfileStringModerationConfig,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profile_text_moderation_enabled: bool,
    pub profile_text_moderation: AdminProfileStringModerationConfig,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub content_moderation_enabled: bool,
    pub content_moderation: AdminContentModerationConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct AdminProfileStringModerationConfig {
    /// Accept all texts which only have single visible character.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub accept_single_visible_character: bool,
    /// Large language model based moderation.
    /// Actions: reject (or move_to_human) and accept
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub llm_enabled: bool,
    pub llm: LlmStringModerationConfig,
    pub default_action: ModerationAction,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LlmStringModerationConfig {
    pub system_text: String,
    /// Placeholder "{text}" is replaced with text which will be
    /// moderated.
    pub user_text_template: String,
    /// If LLM response starts with this text or the first
    /// line of the response contains this text, the profile text
    /// is moderated as accepted. The comparisons are case insensitive.
    pub expected_response: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub move_rejected_to_human_moderation: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub add_llm_output_to_user_visible_rejection_details: bool,
    pub max_tokens: u32,
}

impl LlmStringModerationConfig {
    pub const TEMPLATE_PLACEHOLDER_TEXT: &'static str = "{text}";
}

impl Default for LlmStringModerationConfig {
    fn default() -> Self {
        Self {
            system_text: "You are a dating app text moderator. Output 'accepted' when the text is safe for a dating app. Output 'rejected' when it's not.".to_string(),
            user_text_template: "Text:\n\n{text}".to_string(),
            expected_response: "accepted".to_string(),
            move_rejected_to_human_moderation: false,
            add_llm_output_to_user_visible_rejection_details: false,
            max_tokens: MAX_TOKENS_DEFAULT,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct AdminContentModerationConfig {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub initial_content: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub added_content: bool,
    /// Neural network based detection.
    /// Actions: reject, move_to_human, accept and delete.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub nsfw_detection_enabled: bool,
    pub nsfw_detection: AdminNsfwDetectionConfig,
    /// Large language model based moderation.
    /// Actions: reject (can be replaced with move_to_human or ignore) and
    ///          accept (can be replaced with move_to_human or delete).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub llm_primary_enabled: bool,
    pub llm_primary: LlmContentModerationConfig,
    /// The secondary LLM moderation will run if primary results with ignore
    /// action.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub llm_secondary_enabled: bool,
    pub llm_secondary: LlmContentModerationConfig,
    pub default_action: ModerationAction,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct AdminNsfwDetectionConfig {
    /// Thresholds for image rejection.
    pub reject: simple_backend_model::NsfwDetectionThresholds,
    /// Thresholds for moving image to human moderation.
    pub move_to_human: simple_backend_model::NsfwDetectionThresholds,
    /// Thresholds for accepting the image.
    pub accept: simple_backend_model::NsfwDetectionThresholds,
    /// Thresholds for image deletion.
    pub delete: simple_backend_model::NsfwDetectionThresholds,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LlmContentModerationConfig {
    pub system_text: String,
    /// If LLM response starts with this text or the first
    /// line of the response contains this text, the content
    /// is moderated as accepted. The comparisons are case insensitive.
    pub expected_response: String,
    /// Overrides [Self::move_rejected_to_human_moderation]
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub ignore_rejected: bool,
    /// Overrides [Self::move_accepted_to_human_moderation]
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub delete_accepted: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub move_accepted_to_human_moderation: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub move_rejected_to_human_moderation: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub add_llm_output_to_user_visible_rejection_details: bool,
    pub max_tokens: u32,
}

impl LlmContentModerationConfig {
    pub fn max_tokens(&self) -> u32 {
        self.max_tokens
    }
}

impl Default for LlmContentModerationConfig {
    fn default() -> Self {
        Self {
            system_text: "You are a dating app image moderator. Output 'accepted' when the image is safe for a dating app. Output 'rejected' when it's not.".to_string(),
            expected_response: "accepted".to_string(),
            ignore_rejected: false,
            delete_accepted: false,
            move_accepted_to_human_moderation: false,
            move_rejected_to_human_moderation: false,
            add_llm_output_to_user_visible_rejection_details: false,
            max_tokens: MAX_TOKENS_DEFAULT,
        }
    }
}
