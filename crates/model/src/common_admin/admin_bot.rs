use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const MAX_TOKENS_DEFAULT: u32 = 10_000;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub enum ModerationAction {
    Accept,
    Reject,
    #[default]
    MoveToHuman,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct AdminBotConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_name_moderation: Option<AdminProfileStringModerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_text_moderation: Option<AdminProfileStringModerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_moderation: Option<AdminContentModerationConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct AdminProfileStringModerationConfig {
    /// Accept all texts which only have single visible character.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub accept_single_visible_character: bool,
    /// Large language model based moderation.
    /// Actions: reject (or move_to_human) and accept
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm: Option<LlmStringModerationConfig>,
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
    pub move_rejected_to_human_moderation: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub add_llm_output_to_user_visible_rejection_details: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

impl LlmStringModerationConfig {
    pub const TEMPLATE_PLACEHOLDER_TEXT: &'static str = "{text}";

    pub fn max_tokens(&self) -> u32 {
        self.max_tokens.unwrap_or(MAX_TOKENS_DEFAULT)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AdminContentModerationConfig {
    pub initial_content: bool,
    pub added_content: bool,
    /// Neural network based detection.
    /// Actions: reject, move_to_human, accept and delete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw_detection: Option<AdminNsfwDetectionConfig>,
    /// Large language model based moderation.
    /// Actions: reject (can be replaced with move_to_human or ignore) and
    ///          accept (can be replaced with move_to_human or delete).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_primary: Option<LlmContentModerationConfig>,
    /// The secondary LLM moderation will run if primary results with ignore
    /// action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_secondary: Option<LlmContentModerationConfig>,
    pub default_action: ModerationAction,
}

impl Default for AdminContentModerationConfig {
    fn default() -> Self {
        Self {
            initial_content: true,
            added_content: true,
            nsfw_detection: None,
            llm_primary: None,
            llm_secondary: None,
            default_action: ModerationAction::MoveToHuman,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AdminNsfwDetectionConfig {
    /// Thresholds for image rejection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject: Option<simple_backend_model::NsfwDetectionThresholds>,
    /// Thresholds for moving image to human moderation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_to_human: Option<simple_backend_model::NsfwDetectionThresholds>,
    /// Thresholds for accepting the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept: Option<simple_backend_model::NsfwDetectionThresholds>,
    /// Thresholds for image deletion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<simple_backend_model::NsfwDetectionThresholds>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

impl LlmContentModerationConfig {
    pub fn max_tokens(&self) -> u32 {
        self.max_tokens.unwrap_or(MAX_TOKENS_DEFAULT)
    }
}
