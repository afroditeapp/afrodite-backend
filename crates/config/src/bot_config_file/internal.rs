use std::path::PathBuf;

use model::common_admin::{
    AdminAccountVerificationConfig, AdminBotConfig, AdminContentModerationConfig,
    AdminFaceVerificationConfig, AdminNsfwDetectionConfig, AdminProfileStringModerationConfig,
    LlmContentModerationConfig as AdminLlmContentModerationConfig,
    LlmFaceVerificationConfig as AdminLlmFaceVerificationConfig,
    LlmSecurityContentVerificationConfig as AdminLlmSecurityContentVerificationConfig,
    LlmStringModerationConfig as AdminLlmStringModerationConfig,
};
pub use model::common_admin::{ModerationAction, VerificationAction};
use serde::{Deserialize, Serialize};
pub use simple_backend_model::NsfwDetectionThresholds;
use url::Url;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileStringModerationConfig {
    pub accept_single_visible_character: bool,
    pub llm: Option<LlmStringModerationConfig>,
    pub default_action: ModerationAction,
    pub concurrency: u8,
}

impl ProfileStringModerationConfig {
    pub fn new(
        db: AdminProfileStringModerationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::ProfileStringModerationFileConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file.unwrap_or_default();

        Some(Self {
            accept_single_visible_character: db.accept_single_visible_character,
            llm: LlmStringModerationConfig::new(db.llm, db.llm_enabled, file.llm),
            default_action: db.default_action,
            concurrency: file.concurrency,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmStringModerationConfig {
    pub openai_api_url: Url,
    pub model: String,
    pub temperature: Option<f32>,
    pub seed: Option<i64>,
    pub system_text: String,
    pub user_text_template: String,
    pub expected_response: String,
    pub move_rejected_to_human_moderation: bool,
    pub add_llm_output_to_user_visible_rejection_details: bool,
    pub debug_log_results: bool,
    pub max_tokens: u32,
    pub retry_wait_times_in_seconds: Vec<u16>,
}

impl LlmStringModerationConfig {
    pub const TEMPLATE_PLACEHOLDER_TEXT: &'static str = "{text}";

    pub fn new(
        db: AdminLlmStringModerationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::LlmStringModerationFileConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        Some(Self {
            openai_api_url: file.openai_api_url,
            model: file.model,
            temperature: file.temperature,
            seed: file.seed,
            system_text: db.system_text,
            user_text_template: db.user_text_template,
            expected_response: db.expected_response,
            move_rejected_to_human_moderation: db.move_rejected_to_human_moderation,
            add_llm_output_to_user_visible_rejection_details: db
                .add_llm_output_to_user_visible_rejection_details,
            debug_log_results: file.debug_log_results,
            max_tokens: db.max_tokens,
            retry_wait_times_in_seconds: file.retry_wait_times_in_seconds,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContentModerationConfig {
    pub initial_content: bool,
    pub added_content: bool,
    pub nsfw_detection: Option<NsfwDetectionConfig>,
    pub llm_primary: Option<LlmContentModerationConfig>,
    pub llm_secondary: Option<LlmContentModerationConfig>,
    pub default_action: ModerationAction,
    pub debug_log_delete: bool,
    pub concurrency: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FaceVerificationConfig {
    pub llm: Option<LlmFaceVerificationConfig>,
    pub default_action: VerificationAction,
    pub concurrency: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountVerificationConfig {
    pub profile_age_range: bool,
    pub profile_name: bool,
    pub security_content: Option<SecurityContentVerificationConfig>,
}

impl AccountVerificationConfig {
    pub fn new(
        db: AdminAccountVerificationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::AccountVerificationFileConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file.unwrap_or_default();

        Some(Self {
            profile_age_range: db.profile_age_range_enabled,
            profile_name: db.profile_name_enabled,
            security_content: SecurityContentVerificationConfig::new(
                db.security_content,
                db.security_content_enabled,
                file.security_content,
            ),
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityContentVerificationConfig {
    pub llm: Option<LlmSecurityContentVerificationConfig>,
    pub default_action: VerificationAction,
    pub concurrency: u8,
}

impl SecurityContentVerificationConfig {
    pub fn new(
        db: model::common_admin::AdminSecurityContentVerificationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::SecurityContentVerificationFileConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file.unwrap_or_default();

        Some(Self {
            llm: LlmSecurityContentVerificationConfig::new(db.llm, db.llm_enabled, file.llm),
            default_action: db.default_action,
            concurrency: file.concurrency,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmSecurityContentVerificationConfig {
    pub openai_api_url: Url,
    pub model: String,
    pub temperature: Option<f32>,
    pub seed: Option<i64>,
    pub system_text: String,
    pub expected_response: String,
    pub debug_log_results: bool,
    pub max_tokens: u32,
    pub retry_wait_times_in_seconds: Vec<u16>,
}

impl LlmSecurityContentVerificationConfig {
    pub fn new(
        db: AdminLlmSecurityContentVerificationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::LlmContentModerationFileConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        Some(Self {
            openai_api_url: file.openai_api_url,
            model: file.model,
            temperature: file.temperature,
            seed: file.seed,
            system_text: db.system_text,
            expected_response: db.expected_response,
            debug_log_results: file.debug_log_results,
            max_tokens: db.max_tokens,
            retry_wait_times_in_seconds: file.retry_wait_times_in_seconds,
        })
    }
}

impl FaceVerificationConfig {
    pub fn new(
        db: AdminFaceVerificationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::FaceVerificationFileConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file.unwrap_or_default();

        Some(Self {
            llm: LlmFaceVerificationConfig::new(db.llm, db.llm_enabled, file.llm),
            default_action: db.default_action,
            concurrency: file.concurrency,
        })
    }
}

impl ContentModerationConfig {
    pub fn new(
        db: AdminContentModerationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::ContentModerationFileConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file.unwrap_or_default();

        Some(Self {
            initial_content: db.initial_content,
            added_content: db.added_content,
            nsfw_detection: NsfwDetectionConfig::new(
                db.nsfw_detection,
                db.nsfw_detection_enabled,
                file.nsfw_detection,
            ),
            llm_primary: LlmContentModerationConfig::new(
                db.llm_primary,
                db.llm_primary_enabled,
                file.llm_primary,
            ),
            llm_secondary: LlmContentModerationConfig::new(
                db.llm_secondary,
                db.llm_secondary_enabled,
                file.llm_secondary,
            ),
            default_action: db.default_action,
            debug_log_delete: file.debug_log_delete,
            concurrency: file.concurrency,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NsfwDetectionConfig {
    pub model_file: PathBuf,
    pub reject: NsfwDetectionThresholds,
    pub move_to_human: NsfwDetectionThresholds,
    pub accept: NsfwDetectionThresholds,
    pub delete: NsfwDetectionThresholds,
    pub debug_log_results: bool,
}

impl NsfwDetectionConfig {
    pub fn new(
        db: AdminNsfwDetectionConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::NsfwDetectionFileConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        Some(Self {
            model_file: file.model_file,
            reject: db.reject,
            move_to_human: db.move_to_human,
            accept: db.accept,
            delete: db.delete,
            debug_log_results: file.debug_log_results,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmContentModerationConfig {
    pub openai_api_url: Url,
    pub model: String,
    pub temperature: Option<f32>,
    pub seed: Option<i64>,
    pub system_text: String,
    pub expected_response: String,
    pub ignore_rejected: bool,
    pub delete_accepted: bool,
    pub move_accepted_to_human_moderation: bool,
    pub move_rejected_to_human_moderation: bool,
    pub add_llm_output_to_user_visible_rejection_details: bool,
    pub debug_log_results: bool,
    pub max_tokens: u32,
    pub retry_wait_times_in_seconds: Vec<u16>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmFaceVerificationConfig {
    pub openai_api_url: Url,
    pub model: String,
    pub temperature: Option<f32>,
    pub seed: Option<i64>,
    pub system_text: String,
    pub expected_response: String,
    pub debug_log_results: bool,
    pub max_tokens: u32,
    pub retry_wait_times_in_seconds: Vec<u16>,
}

impl LlmFaceVerificationConfig {
    pub fn new(
        db: AdminLlmFaceVerificationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::LlmContentModerationFileConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        Some(Self {
            openai_api_url: file.openai_api_url,
            model: file.model,
            temperature: file.temperature,
            seed: file.seed,
            system_text: db.system_text,
            expected_response: db.expected_response,
            debug_log_results: file.debug_log_results,
            max_tokens: db.max_tokens,
            retry_wait_times_in_seconds: file.retry_wait_times_in_seconds,
        })
    }
}

impl LlmContentModerationConfig {
    pub fn new(
        db: AdminLlmContentModerationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::LlmContentModerationFileConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        Some(Self {
            openai_api_url: file.openai_api_url,
            model: file.model,
            temperature: file.temperature,
            seed: file.seed,
            system_text: db.system_text,
            expected_response: db.expected_response,
            ignore_rejected: db.ignore_rejected,
            delete_accepted: db.delete_accepted,
            move_accepted_to_human_moderation: db.move_accepted_to_human_moderation,
            move_rejected_to_human_moderation: db.move_rejected_to_human_moderation,
            add_llm_output_to_user_visible_rejection_details: db
                .add_llm_output_to_user_visible_rejection_details,
            debug_log_results: file.debug_log_results,
            max_tokens: db.max_tokens,
            retry_wait_times_in_seconds: file.retry_wait_times_in_seconds,
        })
    }
}

#[allow(clippy::type_complexity)]
pub fn merge(
    db: AdminBotConfig,
    file: crate::bot_config_file::BotConfigFile,
) -> (
    Option<ProfileStringModerationConfig>,
    Option<ProfileStringModerationConfig>,
    Option<ContentModerationConfig>,
    Option<FaceVerificationConfig>,
    Option<AccountVerificationConfig>,
) {
    let name = ProfileStringModerationConfig::new(
        db.profile_name_moderation,
        db.profile_name_moderation_enabled,
        file.profile_name_moderation,
    );
    let text = ProfileStringModerationConfig::new(
        db.profile_text_moderation,
        db.profile_text_moderation_enabled,
        file.profile_text_moderation,
    );
    let content = ContentModerationConfig::new(
        db.content_moderation,
        db.content_moderation_enabled,
        file.content_moderation,
    );
    let face_verification = FaceVerificationConfig::new(
        db.face_verification,
        db.face_verification_enabled,
        file.face_verification,
    );
    let account_verification = AccountVerificationConfig::new(
        db.account_verification,
        db.account_verification_enabled,
        file.account_verification,
    );
    (name, text, content, face_verification, account_verification)
}
