use std::path::PathBuf;

pub use model::common_admin::{AcceptOrReject, ModerationAction};
use model::common_admin::{
    AdminAccountVerificationConfig, AdminBotConfig, AdminContentModerationConfig,
    AdminFaceVerificationConfig, AdminNsfwDetectionConfig, AdminProfileStringModerationConfig,
    AdminReportProcessingConfig, LlmContentModerationConfig as AdminLlmContentModerationConfig,
    LlmFaceVerificationConfig as AdminLlmFaceVerificationConfig,
    LlmSecurityContentVerificationConfig as AdminLlmSecurityContentVerificationConfig,
    LlmStringModerationConfig as AdminLlmStringModerationConfig,
};
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
        common_llm: Option<crate::bot_config_file::LlmConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file.unwrap_or_default();

        Some(Self {
            accept_single_visible_character: db.accept_single_visible_character,
            llm: LlmStringModerationConfig::new(db.llm, db.llm_enabled, file.llm, common_llm),
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
        common_llm: Option<crate::bot_config_file::LlmConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;
        let llm = match file.llm {
            Some(opt) => Some(opt.merge_with(common_llm?)),
            None => common_llm,
        }?;

        Some(Self {
            openai_api_url: llm.openai_api_url,
            model: llm.model,
            temperature: llm.temperature,
            seed: llm.seed,
            system_text: db.system_text,
            user_text_template: db.user_text_template,
            expected_response: db.expected_response,
            move_rejected_to_human_moderation: db.move_rejected_to_human_moderation,
            add_llm_output_to_user_visible_rejection_details: db
                .add_llm_output_to_user_visible_rejection_details,
            debug_log_results: llm.debug_log_results,
            max_tokens: llm.max_tokens,
            retry_wait_times_in_seconds: llm.retry_wait_times_in_seconds,
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
    pub default_action: AcceptOrReject,
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
        common_llm: Option<crate::bot_config_file::LlmConfig>,
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
                common_llm,
            ),
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityContentVerificationConfig {
    pub llm: Option<LlmSecurityContentVerificationConfig>,
    pub default_action: AcceptOrReject,
    pub concurrency: u8,
}

impl SecurityContentVerificationConfig {
    pub fn new(
        db: model::common_admin::AdminSecurityContentVerificationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::SecurityContentVerificationFileConfig>,
        common_llm: Option<crate::bot_config_file::LlmConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file.unwrap_or_default();

        Some(Self {
            llm: LlmSecurityContentVerificationConfig::new(
                db.llm,
                db.llm_enabled,
                file.llm,
                common_llm,
            ),
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
        common_llm: Option<crate::bot_config_file::LlmConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;
        let llm = match file.llm {
            Some(opt) => Some(opt.merge_with(common_llm?)),
            None => common_llm,
        }?;

        Some(Self {
            openai_api_url: llm.openai_api_url,
            model: llm.model,
            temperature: llm.temperature,
            seed: llm.seed,
            system_text: db.system_text,
            expected_response: db.expected_response,
            debug_log_results: llm.debug_log_results,
            max_tokens: llm.max_tokens,
            retry_wait_times_in_seconds: llm.retry_wait_times_in_seconds,
        })
    }
}

impl FaceVerificationConfig {
    pub fn new(
        db: AdminFaceVerificationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::FaceVerificationFileConfig>,
        common_llm: Option<crate::bot_config_file::LlmConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file.unwrap_or_default();

        Some(Self {
            llm: LlmFaceVerificationConfig::new(db.llm, db.llm_enabled, file.llm, common_llm),
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
        common_llm: Option<crate::bot_config_file::LlmConfig>,
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
                common_llm.clone(),
            ),
            llm_secondary: LlmContentModerationConfig::new(
                db.llm_secondary,
                db.llm_secondary_enabled,
                file.llm_secondary,
                common_llm,
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
        common_llm: Option<crate::bot_config_file::LlmConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;
        let llm = match file.llm {
            Some(opt) => Some(opt.merge_with(common_llm?)),
            None => common_llm,
        }?;

        Some(Self {
            openai_api_url: llm.openai_api_url,
            model: llm.model,
            temperature: llm.temperature,
            seed: llm.seed,
            system_text: db.system_text,
            expected_response: db.expected_response,
            debug_log_results: llm.debug_log_results,
            max_tokens: llm.max_tokens,
            retry_wait_times_in_seconds: llm.retry_wait_times_in_seconds,
        })
    }
}

impl LlmContentModerationConfig {
    pub fn new(
        db: AdminLlmContentModerationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::LlmContentModerationFileConfig>,
        common_llm: Option<crate::bot_config_file::LlmConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;
        let llm = match file.llm {
            Some(opt) => Some(opt.merge_with(common_llm?)),
            None => common_llm,
        }?;

        Some(Self {
            openai_api_url: llm.openai_api_url,
            model: llm.model,
            temperature: llm.temperature,
            seed: llm.seed,
            system_text: db.system_text,
            expected_response: db.expected_response,
            ignore_rejected: db.ignore_rejected,
            delete_accepted: db.delete_accepted,
            move_accepted_to_human_moderation: db.move_accepted_to_human_moderation,
            move_rejected_to_human_moderation: db.move_rejected_to_human_moderation,
            add_llm_output_to_user_visible_rejection_details: db
                .add_llm_output_to_user_visible_rejection_details,
            debug_log_results: llm.debug_log_results,
            max_tokens: llm.max_tokens,
            retry_wait_times_in_seconds: llm.retry_wait_times_in_seconds,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmReportProcessingConfig {
    pub openai_api_url: Url,
    pub model: String,
    pub temperature: Option<f32>,
    pub seed: Option<i64>,
    pub system_text: String,
    pub user_text_template: Option<String>,
    pub report_creator_message_template: Option<String>,
    pub report_target_message_template: Option<String>,
    pub expected_response: String,
    pub debug_log_results: bool,
    pub max_tokens: u32,
    pub retry_wait_times_in_seconds: Vec<u16>,
}

impl LlmReportProcessingConfig {
    pub const TEMPLATE_PLACEHOLDER_TEXT: &'static str = "{text}";

    pub fn from_db_profile_string(
        db: model::common_admin::AdminReportProcessingProfileStringLlmConfig,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        common_llm: Option<crate::bot_config_file::LlmConfig>,
    ) -> Option<Self> {
        let file = file?;
        let llm = match file.llm {
            Some(opt) => Some(opt.merge_with(common_llm?)),
            None => common_llm,
        }?;

        Some(Self {
            openai_api_url: llm.openai_api_url,
            model: llm.model,
            temperature: llm.temperature,
            seed: llm.seed,
            system_text: db.system_text,
            user_text_template: Some(db.user_text_template),
            report_creator_message_template: None,
            report_target_message_template: None,
            expected_response: db.expected_response,
            debug_log_results: llm.debug_log_results,
            max_tokens: llm.max_tokens,
            retry_wait_times_in_seconds: llm.retry_wait_times_in_seconds,
        })
    }

    pub fn from_db_profile_content(
        db: model::common_admin::AdminReportProcessingProfileContentLlmConfig,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        common_llm: Option<crate::bot_config_file::LlmConfig>,
    ) -> Option<Self> {
        let file = file?;
        let llm = match file.llm {
            Some(opt) => Some(opt.merge_with(common_llm?)),
            None => common_llm,
        }?;

        Some(Self {
            openai_api_url: llm.openai_api_url,
            model: llm.model,
            temperature: llm.temperature,
            seed: llm.seed,
            system_text: db.system_text,
            user_text_template: None,
            report_creator_message_template: None,
            report_target_message_template: None,
            expected_response: db.expected_response,
            debug_log_results: llm.debug_log_results,
            max_tokens: llm.max_tokens,
            retry_wait_times_in_seconds: llm.retry_wait_times_in_seconds,
        })
    }

    pub fn from_db_messages(
        db: model::common_admin::AdminReportProcessingMessagesLlmConfig,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        common_llm: Option<crate::bot_config_file::LlmConfig>,
    ) -> Option<Self> {
        let file = file?;
        let llm = match file.llm {
            Some(opt) => Some(opt.merge_with(common_llm?)),
            None => common_llm,
        }?;

        Some(Self {
            openai_api_url: llm.openai_api_url,
            model: llm.model,
            temperature: llm.temperature,
            seed: llm.seed,
            system_text: db.system_text,
            user_text_template: Some(db.user_text_template),
            report_creator_message_template: Some(db.report_creator_message_template),
            report_target_message_template: Some(db.report_target_message_template),
            expected_response: db.expected_response,
            debug_log_results: llm.debug_log_results,
            max_tokens: llm.max_tokens,
            retry_wait_times_in_seconds: llm.retry_wait_times_in_seconds,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReportProcessingTypeConfig {
    pub llm: Option<LlmReportProcessingConfig>,
    pub default_action: AcceptOrReject,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReportProcessingConfig {
    pub profile_name: Option<ReportProcessingTypeConfig>,
    pub profile_text: Option<ReportProcessingTypeConfig>,
    pub profile_content: Option<ReportProcessingTypeConfig>,
    pub messages: Option<ReportProcessingTypeConfig>,
    pub concurrency: u8,
}

impl ReportProcessingConfig {
    fn new_per_type_profile_string(
        db_llm: model::common_admin::AdminReportProcessingProfileStringLlmConfig,
        db_enabled: bool,
        default_action: AcceptOrReject,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        common_llm: Option<crate::bot_config_file::LlmConfig>,
    ) -> Option<ReportProcessingTypeConfig> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        Some(ReportProcessingTypeConfig {
            llm: LlmReportProcessingConfig::from_db_profile_string(db_llm, Some(file), common_llm),
            default_action,
        })
    }

    fn new_per_type_profile_content(
        db_llm: model::common_admin::AdminReportProcessingProfileContentLlmConfig,
        db_enabled: bool,
        default_action: AcceptOrReject,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        common_llm: Option<crate::bot_config_file::LlmConfig>,
    ) -> Option<ReportProcessingTypeConfig> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        Some(ReportProcessingTypeConfig {
            llm: LlmReportProcessingConfig::from_db_profile_content(db_llm, Some(file), common_llm),
            default_action,
        })
    }

    fn new_per_type_messages(
        db_llm: model::common_admin::AdminReportProcessingMessagesLlmConfig,
        db_enabled: bool,
        default_action: AcceptOrReject,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        common_llm: Option<crate::bot_config_file::LlmConfig>,
    ) -> Option<ReportProcessingTypeConfig> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        Some(ReportProcessingTypeConfig {
            llm: LlmReportProcessingConfig::from_db_messages(db_llm, Some(file), common_llm),
            default_action,
        })
    }

    pub fn new(
        db: AdminReportProcessingConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::ReportProcessingFileConfig>,
        common_llm: Option<crate::bot_config_file::LlmConfig>,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        Some(Self {
            profile_name: Self::new_per_type_profile_string(
                db.profile_name,
                db.profile_name_enabled,
                db.profile_name_default_action,
                file.profile_name,
                common_llm.clone(),
            ),
            profile_text: Self::new_per_type_profile_string(
                db.profile_text,
                db.profile_text_enabled,
                db.profile_text_default_action,
                file.profile_text,
                common_llm.clone(),
            ),
            profile_content: Self::new_per_type_profile_content(
                db.profile_content,
                db.profile_content_enabled,
                db.profile_content_default_action,
                file.profile_content,
                common_llm.clone(),
            ),
            messages: Self::new_per_type_messages(
                db.messages,
                db.messages_enabled,
                db.messages_default_action,
                file.messages,
                common_llm,
            ),
            concurrency: file.concurrency,
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
    Option<ReportProcessingConfig>,
) {
    let name = ProfileStringModerationConfig::new(
        db.profile_name_moderation,
        db.profile_name_moderation_enabled,
        file.profile_name_moderation,
        file.llm.clone(),
    );
    let text = ProfileStringModerationConfig::new(
        db.profile_text_moderation,
        db.profile_text_moderation_enabled,
        file.profile_text_moderation,
        file.llm.clone(),
    );
    let content = ContentModerationConfig::new(
        db.content_moderation,
        db.content_moderation_enabled,
        file.content_moderation,
        file.llm.clone(),
    );
    let face_verification = FaceVerificationConfig::new(
        db.face_verification,
        db.face_verification_enabled,
        file.face_verification,
        file.llm.clone(),
    );
    let account_verification = AccountVerificationConfig::new(
        db.account_verification,
        db.account_verification_enabled,
        file.account_verification,
        file.llm.clone(),
    );
    let report_processing = ReportProcessingConfig::new(
        db.report_processing,
        db.report_processing_enabled,
        file.report_processing,
        file.llm,
    );
    (
        name,
        text,
        content,
        face_verification,
        account_verification,
        report_processing,
    )
}
