use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, Default)]
pub enum ModerationAction {
    Accept,
    Reject,
    #[default]
    MoveToHuman,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, Default)]
pub enum AcceptOrReject {
    Accept,
    #[default]
    Reject,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AdminBotBaseLlmConfig {
    pub system_text: String,
    /// If LLM response starts with this text or the first
    /// line of the response contains this text, the content
    /// is moderated as accepted. The comparisons are case insensitive.
    pub expected_response: String,
}

impl AdminBotBaseLlmConfig {
    pub const fn new(system_text: String, expected_response: String) -> Self {
        Self {
            system_text,
            expected_response,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AdminBotModerationBaseLlmConfig {
    pub system_text: String,
    /// If LLM response starts with this text or the first
    /// line of the response contains this text, the content
    /// is moderated as accepted. The comparisons are case insensitive.
    pub expected_response: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub add_llm_output_to_user_visible_rejection_details: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub move_rejected_to_human_moderation: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AdminBotStringReportBaseLlmConfig {
    pub system_text: String,
    /// If LLM response starts with this text or the first
    /// line of the response contains this text, the content
    /// is moderated as accepted. The comparisons are case insensitive.
    pub expected_response: String,
    /// Placeholder "{text}" is replaced with the reported content.
    pub user_text_template: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct AdminBotConfig {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profile_name_moderation_enabled: bool,
    pub profile_name_moderation: AdminBotProfileStringModerationConfig,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profile_text_moderation_enabled: bool,
    pub profile_text_moderation: AdminBotProfileStringModerationConfig,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub content_moderation_enabled: bool,
    pub content_moderation: AdminBotContentModerationConfig,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub face_verification_enabled: bool,
    pub face_verification: AdminBotFaceVerificationConfig,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub account_verification_enabled: bool,
    pub account_verification: AdminBotAccountVerificationConfig,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub report_processing_enabled: bool,
    pub report_processing: AdminBotReportProcessingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct AdminBotFaceVerificationConfig {
    /// Large language model based face verification.
    /// Actions: reject and accept.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub llm_enabled: bool,
    pub llm: AdminBotFaceVerificationLlmConfig,
    pub default_action: AcceptOrReject,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AdminBotFaceVerificationLlmConfig {
    #[serde(flatten)]
    pub base: AdminBotBaseLlmConfig,
}

impl Default for AdminBotFaceVerificationLlmConfig {
    fn default() -> Self {
        Self {
            base: AdminBotBaseLlmConfig::new(
                "You are verifying whether two dating app profile images contain the same person. Output 'accepted' only when they clearly show the same person. Otherwise output 'rejected'.".to_string(),
                "accepted".to_string(),
            ),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct AdminBotAccountVerificationConfig {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profile_age_range_enabled: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profile_name_enabled: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub security_content_enabled: bool,
    pub security_content: AdminBotSecurityContentVerificationConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct AdminBotSecurityContentVerificationConfig {
    /// Large language model based security content verification.
    /// Actions: reject and accept.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub llm_enabled: bool,
    pub llm: AdminBotSecurityContentVerificationLlmConfig,
    pub default_action: AcceptOrReject,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AdminBotSecurityContentVerificationLlmConfig {
    #[serde(flatten)]
    pub base: AdminBotBaseLlmConfig,
}

impl Default for AdminBotSecurityContentVerificationLlmConfig {
    fn default() -> Self {
        Self {
            base: AdminBotBaseLlmConfig::new(
                "You are verifying whether a dating app profile security selfie and a user-provided verification image show the same person. Output 'accepted' only when they clearly show the same person. Otherwise output 'rejected'.".to_string(),
                "accepted".to_string(),
            ),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AdminBotReportProcessingProfileStringLlmConfig {
    #[serde(flatten)]
    pub base: AdminBotStringReportBaseLlmConfig,
}

impl Default for AdminBotReportProcessingProfileStringLlmConfig {
    fn default() -> Self {
        Self {
            base: AdminBotStringReportBaseLlmConfig {
                system_text: "You are a dating app text content moderator. Output 'accepted' when the reported text violates terms. Output 'rejected' when it does not.".to_string(),
                expected_response: "accepted".to_string(),
                user_text_template: "Reported content:\n\n{text}".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AdminBotReportProcessingProfileContentLlmConfig {
    #[serde(flatten)]
    pub base: AdminBotBaseLlmConfig,
}

impl Default for AdminBotReportProcessingProfileContentLlmConfig {
    fn default() -> Self {
        Self {
            base: AdminBotBaseLlmConfig::new(
                "You are a dating app image report moderator. Output 'accepted' when the reported image violates terms. Output 'rejected' when it does not.".to_string(),
                "accepted".to_string(),
            ),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AdminBotReportProcessingMessagesLlmConfig {
    #[serde(flatten)]
    pub base: AdminBotStringReportBaseLlmConfig,
    /// Placeholder "{text}" is replaced with the report creator's message.
    pub report_creator_message_template: String,
    /// Placeholder "{text}" is replaced with the report target's message.
    pub report_target_message_template: String,
}

impl Default for AdminBotReportProcessingMessagesLlmConfig {
    fn default() -> Self {
        Self {
            base: AdminBotStringReportBaseLlmConfig {
                system_text: "You are a dating app chat message report moderator. Output 'accepted' when the reported messages violate terms. Output 'rejected' when they do not.".to_string(),
                expected_response: "accepted".to_string(),
                user_text_template: "Reported messages:\n\n{text}".to_string(),
            },
            report_creator_message_template: "Report creator's message:\n\n{text}".to_string(),
            report_target_message_template: "Report target's message:\n\n{text}".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct AdminBotReportProcessingConfig {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profile_name_enabled: bool,
    pub profile_name: AdminBotReportProcessingProfileStringLlmConfig,
    pub profile_name_default_action: AcceptOrReject,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profile_text_enabled: bool,
    pub profile_text: AdminBotReportProcessingProfileStringLlmConfig,
    pub profile_text_default_action: AcceptOrReject,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profile_content_enabled: bool,
    pub profile_content: AdminBotReportProcessingProfileContentLlmConfig,
    pub profile_content_default_action: AcceptOrReject,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub messages_enabled: bool,
    pub messages: AdminBotReportProcessingMessagesLlmConfig,
    pub messages_default_action: AcceptOrReject,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct AdminBotProfileStringModerationConfig {
    /// Accept all texts which only have single visible character.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub accept_single_visible_character: bool,
    /// Large language model based moderation.
    /// Actions: reject (or move_to_human) and accept
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub llm_enabled: bool,
    pub llm: AdminBotStringModerationLlmConfig,
    pub default_action: ModerationAction,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AdminBotStringModerationLlmConfig {
    #[serde(flatten)]
    pub base: AdminBotModerationBaseLlmConfig,
    /// Placeholder "{text}" is replaced with text which will be
    /// moderated.
    pub user_text_template: String,
}

impl Default for AdminBotStringModerationLlmConfig {
    fn default() -> Self {
        Self {
            base: AdminBotModerationBaseLlmConfig {
                system_text: "You are a dating app text moderator. Output 'accepted' when the text is safe for a dating app. Output 'rejected' when it's not.".to_string(),
                expected_response: "accepted".to_string(),
                move_rejected_to_human_moderation: false,
                add_llm_output_to_user_visible_rejection_details: false,
            },
            user_text_template: "Text:\n\n{text}".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct AdminBotContentModerationConfig {
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
    pub nsfw_detection: AdminBotNsfwDetectionConfig,
    /// Large language model based moderation.
    /// Actions: reject (can be replaced with move_to_human or ignore) and
    ///          accept (can be replaced with move_to_human or delete).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub llm_primary_enabled: bool,
    pub llm_primary: AdminBotContentModerationLlmConfig,
    /// The secondary LLM moderation will run if primary results with ignore
    /// action.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub llm_secondary_enabled: bool,
    pub llm_secondary: AdminBotContentModerationLlmConfig,
    pub default_action: ModerationAction,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct AdminBotNsfwDetectionConfig {
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
pub struct AdminBotContentModerationLlmConfig {
    #[serde(flatten)]
    pub base: AdminBotBaseLlmConfig,
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
}

impl Default for AdminBotContentModerationLlmConfig {
    fn default() -> Self {
        Self {
            base: AdminBotBaseLlmConfig::new(
                "You are a dating app image moderator. Output 'accepted' when the image is safe for a dating app. Output 'rejected' when it's not.".to_string(),
                "accepted".to_string(),
            ),
            ignore_rejected: false,
            delete_accepted: false,
            move_accepted_to_human_moderation: false,
            move_rejected_to_human_moderation: false,
            add_llm_output_to_user_visible_rejection_details: false,
        }
    }
}
