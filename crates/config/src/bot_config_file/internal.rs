use std::path::PathBuf;

pub use model::common_admin::{AcceptOrReject, ModerationAction};
use model::common_admin::{
    AdminBotAccountVerificationConfig, AdminBotConfig, AdminBotContentModerationConfig,
    AdminBotContentModerationLlmConfig, AdminBotFaceVerificationConfig,
    AdminBotFaceVerificationLlmConfig, AdminBotNsfwDetectionConfig,
    AdminBotProfileStringModerationConfig, AdminBotReportProcessingConfig,
    AdminBotSecurityContentVerificationLlmConfig, AdminBotStringModerationLlmConfig,
};
pub use simple_backend_model::NsfwDetectionThresholds;

use crate::bot_config_file::LlmConfig;

#[derive(Debug, Clone)]
pub struct ProfileStringModerationConfigInternal {
    pub accept_single_visible_character: bool,
    pub llm: Option<LlmStringModerationConfigInternal>,
    pub default_action: ModerationAction,
    pub concurrency: u8,
}

impl ProfileStringModerationConfigInternal {
    pub fn new(
        db: AdminBotProfileStringModerationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::ProfileStringModerationFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file.unwrap_or_default();

        Some(Self {
            accept_single_visible_character: db.accept_single_visible_character,
            llm: LlmStringModerationConfigInternal::new(db.llm, db.llm_enabled, file.llm, base_llm),
            default_action: db.default_action,
            concurrency: file.concurrency,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LlmStringModerationConfigInternal {
    pub llm: LlmConfig,
    pub user_text_template: String,
    pub system_text: String,
    pub expected_response: String,
    pub move_rejected_to_human_moderation: bool,
    pub add_llm_output_to_user_visible_rejection_details: bool,
}

impl LlmStringModerationConfigInternal {
    pub const TEMPLATE_PLACEHOLDER_TEXT: &'static str = "{text}";

    pub fn new(
        db: AdminBotStringModerationLlmConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::LlmStringModerationFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;
        let llm = file.llm.unwrap_or_default().merge_with(base_llm)?;

        Some(Self {
            llm,
            user_text_template: db.user_text_template,
            system_text: db.system_text,
            expected_response: db.expected_response,
            move_rejected_to_human_moderation: db.move_rejected_to_human_moderation,
            add_llm_output_to_user_visible_rejection_details: db
                .add_llm_output_to_user_visible_rejection_details,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ContentModerationConfigInternal {
    pub initial_content: bool,
    pub added_content: bool,
    pub nsfw_detection: Option<NsfwDetectionConfigInternal>,
    pub llm_primary: Option<LlmContentModerationConfigInternal>,
    pub llm_secondary: Option<LlmContentModerationConfigInternal>,
    pub default_action: ModerationAction,
    pub debug_log_delete: bool,
    pub concurrency: u8,
}

#[derive(Debug, Clone)]
pub struct FaceVerificationConfigInternal {
    pub llm: Option<LlmFaceVerificationConfigInternal>,
    pub default_action: AcceptOrReject,
    pub concurrency: u8,
}

#[derive(Debug, Clone)]
pub struct AccountVerificationConfigInternal {
    pub profile_age_range: bool,
    pub profile_name: bool,
    pub security_content: Option<SecurityContentVerificationConfigInternal>,
}

impl AccountVerificationConfigInternal {
    pub fn new(
        db: AdminBotAccountVerificationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::AccountVerificationFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file.unwrap_or_default();

        Some(Self {
            profile_age_range: db.profile_age_range_enabled,
            profile_name: db.profile_name_enabled,
            security_content: SecurityContentVerificationConfigInternal::new(
                db.security_content,
                db.security_content_enabled,
                file.security_content,
                base_llm,
            ),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SecurityContentVerificationConfigInternal {
    pub llm: Option<LlmSecurityContentVerificationConfigInternal>,
    pub default_action: AcceptOrReject,
    pub concurrency: u8,
}

impl SecurityContentVerificationConfigInternal {
    pub fn new(
        db: model::common_admin::AdminBotSecurityContentVerificationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::SecurityContentVerificationFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file.unwrap_or_default();

        Some(Self {
            llm: LlmSecurityContentVerificationConfigInternal::new(
                db.llm,
                db.llm_enabled,
                file.llm,
                base_llm,
            ),
            default_action: db.default_action,
            concurrency: file.concurrency,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LlmSecurityContentVerificationConfigInternal {
    pub llm: LlmConfig,
    pub system_text: String,
    pub expected_response: String,
}

impl LlmSecurityContentVerificationConfigInternal {
    pub fn new(
        db: AdminBotSecurityContentVerificationLlmConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::LlmContentModerationFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;
        let llm = file.llm.unwrap_or_default().merge_with(base_llm)?;

        Some(Self {
            llm,
            system_text: db.system_text,
            expected_response: db.expected_response,
        })
    }
}

impl FaceVerificationConfigInternal {
    pub fn new(
        db: AdminBotFaceVerificationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::FaceVerificationFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file.unwrap_or_default();

        Some(Self {
            llm: LlmFaceVerificationConfigInternal::new(db.llm, db.llm_enabled, file.llm, base_llm),
            default_action: db.default_action,
            concurrency: file.concurrency,
        })
    }
}

impl ContentModerationConfigInternal {
    pub fn new(
        db: AdminBotContentModerationConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::ContentModerationFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file.unwrap_or_default();

        Some(Self {
            initial_content: db.initial_content,
            added_content: db.added_content,
            nsfw_detection: NsfwDetectionConfigInternal::new(
                db.nsfw_detection,
                db.nsfw_detection_enabled,
                file.nsfw_detection,
            ),
            llm_primary: LlmContentModerationConfigInternal::new(
                db.llm_primary,
                db.llm_primary_enabled,
                file.llm_primary,
                base_llm.clone(),
            ),
            llm_secondary: LlmContentModerationConfigInternal::new(
                db.llm_secondary,
                db.llm_secondary_enabled,
                file.llm_secondary,
                base_llm,
            ),
            default_action: db.default_action,
            debug_log_delete: file.debug_log_delete,
            concurrency: file.concurrency,
        })
    }
}

#[derive(Debug, Clone)]
pub struct NsfwDetectionConfigInternal {
    pub model_file: PathBuf,
    pub reject: NsfwDetectionThresholds,
    pub move_to_human: NsfwDetectionThresholds,
    pub accept: NsfwDetectionThresholds,
    pub delete: NsfwDetectionThresholds,
    pub debug_log_results: bool,
}

impl NsfwDetectionConfigInternal {
    pub fn new(
        db: AdminBotNsfwDetectionConfig,
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

#[derive(Debug, Clone)]
pub struct LlmContentModerationConfigInternal {
    pub llm: LlmConfig,
    pub system_text: String,
    pub expected_response: String,
    pub ignore_rejected: bool,
    pub delete_accepted: bool,
    pub move_accepted_to_human_moderation: bool,
    pub move_rejected_to_human_moderation: bool,
    pub add_llm_output_to_user_visible_rejection_details: bool,
}

#[derive(Debug, Clone)]
pub struct LlmFaceVerificationConfigInternal {
    pub llm: LlmConfig,
    pub system_text: String,
    pub expected_response: String,
}

impl LlmFaceVerificationConfigInternal {
    pub fn new(
        db: AdminBotFaceVerificationLlmConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::LlmContentModerationFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;
        let llm = file.llm.unwrap_or_default().merge_with(base_llm)?;

        Some(Self {
            llm,
            system_text: db.system_text,
            expected_response: db.expected_response,
        })
    }
}

impl LlmContentModerationConfigInternal {
    pub fn new(
        db: AdminBotContentModerationLlmConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::LlmContentModerationFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;
        let llm = file.llm.unwrap_or_default().merge_with(base_llm)?;

        Some(Self {
            llm,
            system_text: db.system_text,
            expected_response: db.expected_response,
            ignore_rejected: db.ignore_rejected,
            delete_accepted: db.delete_accepted,
            move_accepted_to_human_moderation: db.move_accepted_to_human_moderation,
            move_rejected_to_human_moderation: db.move_rejected_to_human_moderation,
            add_llm_output_to_user_visible_rejection_details: db
                .add_llm_output_to_user_visible_rejection_details,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LlmReportProcessingConfigInternal {
    pub llm: LlmConfig,
    pub system_text: String,
    pub user_text_template: Option<String>,
    pub report_creator_message_template: Option<String>,
    pub report_target_message_template: Option<String>,
    pub expected_response: String,
}

impl LlmReportProcessingConfigInternal {
    pub const TEMPLATE_PLACEHOLDER_TEXT: &'static str = "{text}";

    pub fn from_db_profile_string(
        db: model::common_admin::AdminBotReportProcessingProfileStringLlmConfig,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        let file = file?;
        let llm = file.llm.unwrap_or_default().merge_with(base_llm)?;

        Some(Self {
            llm,
            system_text: db.system_text,
            user_text_template: Some(db.user_text_template),
            report_creator_message_template: None,
            report_target_message_template: None,
            expected_response: db.expected_response,
        })
    }

    pub fn from_db_profile_content(
        db: model::common_admin::AdminBotReportProcessingProfileContentLlmConfig,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        let file = file?;
        let llm = file.llm.unwrap_or_default().merge_with(base_llm)?;

        Some(Self {
            llm,
            system_text: db.system_text,
            user_text_template: None,
            report_creator_message_template: None,
            report_target_message_template: None,
            expected_response: db.expected_response,
        })
    }

    pub fn from_db_messages(
        db: model::common_admin::AdminBotReportProcessingMessagesLlmConfig,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        let file = file?;
        let llm = file.llm.unwrap_or_default().merge_with(base_llm)?;

        Some(Self {
            llm,
            system_text: db.system_text,
            user_text_template: Some(db.user_text_template),
            report_creator_message_template: Some(db.report_creator_message_template),
            report_target_message_template: Some(db.report_target_message_template),
            expected_response: db.expected_response,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ReportProcessingTypeConfigInternal {
    pub llm: Option<LlmReportProcessingConfigInternal>,
    pub default_action: AcceptOrReject,
}

#[derive(Debug, Clone)]
pub struct ReportProcessingConfigInternal {
    pub profile_name: Option<ReportProcessingTypeConfigInternal>,
    pub profile_text: Option<ReportProcessingTypeConfigInternal>,
    pub profile_content: Option<ReportProcessingTypeConfigInternal>,
    pub messages: Option<ReportProcessingTypeConfigInternal>,
    pub concurrency: u8,
}

impl ReportProcessingConfigInternal {
    fn new_per_type_profile_string(
        db_llm: model::common_admin::AdminBotReportProcessingProfileStringLlmConfig,
        db_enabled: bool,
        default_action: AcceptOrReject,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<ReportProcessingTypeConfigInternal> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        Some(ReportProcessingTypeConfigInternal {
            llm: LlmReportProcessingConfigInternal::from_db_profile_string(
                db_llm,
                Some(file),
                base_llm,
            ),
            default_action,
        })
    }

    fn new_per_type_profile_content(
        db_llm: model::common_admin::AdminBotReportProcessingProfileContentLlmConfig,
        db_enabled: bool,
        default_action: AcceptOrReject,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<ReportProcessingTypeConfigInternal> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        Some(ReportProcessingTypeConfigInternal {
            llm: LlmReportProcessingConfigInternal::from_db_profile_content(
                db_llm,
                Some(file),
                base_llm,
            ),
            default_action,
        })
    }

    fn new_per_type_messages(
        db_llm: model::common_admin::AdminBotReportProcessingMessagesLlmConfig,
        db_enabled: bool,
        default_action: AcceptOrReject,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<ReportProcessingTypeConfigInternal> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        Some(ReportProcessingTypeConfigInternal {
            llm: LlmReportProcessingConfigInternal::from_db_messages(db_llm, Some(file), base_llm),
            default_action,
        })
    }

    pub fn new(
        db: AdminBotReportProcessingConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::ReportProcessingFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
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
                base_llm.clone(),
            ),
            profile_text: Self::new_per_type_profile_string(
                db.profile_text,
                db.profile_text_enabled,
                db.profile_text_default_action,
                file.profile_text,
                base_llm.clone(),
            ),
            profile_content: Self::new_per_type_profile_content(
                db.profile_content,
                db.profile_content_enabled,
                db.profile_content_default_action,
                file.profile_content,
                base_llm.clone(),
            ),
            messages: Self::new_per_type_messages(
                db.messages,
                db.messages_enabled,
                db.messages_default_action,
                file.messages,
                base_llm,
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
    Option<ProfileStringModerationConfigInternal>,
    Option<ProfileStringModerationConfigInternal>,
    Option<ContentModerationConfigInternal>,
    Option<FaceVerificationConfigInternal>,
    Option<AccountVerificationConfigInternal>,
    Option<ReportProcessingConfigInternal>,
) {
    let base_llm = file.llm.unwrap_or_default();

    let name = ProfileStringModerationConfigInternal::new(
        db.profile_name_moderation,
        db.profile_name_moderation_enabled,
        file.profile_name_moderation,
        base_llm.clone(),
    );
    let text = ProfileStringModerationConfigInternal::new(
        db.profile_text_moderation,
        db.profile_text_moderation_enabled,
        file.profile_text_moderation,
        base_llm.clone(),
    );
    let content = ContentModerationConfigInternal::new(
        db.content_moderation,
        db.content_moderation_enabled,
        file.content_moderation,
        base_llm.clone(),
    );
    let face_verification = FaceVerificationConfigInternal::new(
        db.face_verification,
        db.face_verification_enabled,
        file.face_verification,
        base_llm.clone(),
    );
    let account_verification = AccountVerificationConfigInternal::new(
        db.account_verification,
        db.account_verification_enabled,
        file.account_verification,
        base_llm.clone(),
    );
    let report_processing = ReportProcessingConfigInternal::new(
        db.report_processing,
        db.report_processing_enabled,
        file.report_processing,
        base_llm,
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
