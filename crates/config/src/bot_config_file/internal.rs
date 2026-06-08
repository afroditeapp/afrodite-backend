use std::path::PathBuf;

pub use model::common_admin::{AcceptOrReject, ModerationAction};
use model::common_admin::{
    AdminBotAccountVerificationConfig, AdminBotConfig, AdminBotContentModerationConfig,
    AdminBotContentModerationLlmConfig, AdminBotFaceVerificationConfig,
    AdminBotFaceVerificationLlmConfig, AdminBotNsfwDetectionConfig,
    AdminBotProfileStringModerationConfig, AdminBotReportProcessingConfig,
    AdminBotReportProcessingMessagesLlmConfig, AdminBotReportProcessingProfileContentLlmConfig,
    AdminBotReportProcessingProfileStringLlmConfig, AdminBotSecurityContentVerificationLlmConfig,
    AdminBotStringModerationLlmConfig,
};
pub use simple_backend_model::NsfwDetectionThresholds;

use crate::bot_config_file::LlmConfig;

const TEMPLATE_PLACEHOLDER_TEXT: &str = "{text}";

#[derive(Debug, Clone)]
pub struct ProfileStringModerationConfigInternal {
    pub accept_single_visible_character: bool,
    pub llm: Option<ProfileStringModerationLlmConfigInternal>,
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
            llm: ProfileStringModerationLlmConfigInternal::new(
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
pub struct ProfileStringModerationLlmConfigInternal {
    pub db: AdminBotStringModerationLlmConfig,
    pub llm: LlmConfig,
}

impl ProfileStringModerationLlmConfigInternal {
    pub const TEMPLATE_PLACEHOLDER_TEXT: &str = TEMPLATE_PLACEHOLDER_TEXT;

    pub fn new(
        db: AdminBotStringModerationLlmConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::ProfileStringModerationLlmFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;
        let llm = file.llm.unwrap_or_default().merge_with(base_llm)?;

        Some(Self { db, llm })
    }
}

#[derive(Debug, Clone)]
pub struct ContentModerationConfigInternal {
    pub initial_content: bool,
    pub added_content: bool,
    pub nsfw_detection: Option<NsfwDetectionConfigInternal>,
    pub llm_primary: Option<ContentModerationLlmConfigInternal>,
    pub llm_secondary: Option<ContentModerationLlmConfigInternal>,
    pub default_action: ModerationAction,
    pub debug_log_delete: bool,
    pub concurrency: u8,
}

#[derive(Debug, Clone)]
pub struct FaceVerificationConfigInternal {
    pub llm: Option<FaceVerificationLlmConfigInternal>,
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
    pub llm: Option<SecurityContentVerificationLlmConfigInternal>,
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
            llm: SecurityContentVerificationLlmConfigInternal::new(
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
pub struct SecurityContentVerificationLlmConfigInternal {
    pub db: AdminBotSecurityContentVerificationLlmConfig,
    pub llm: LlmConfig,
}

impl SecurityContentVerificationLlmConfigInternal {
    pub fn new(
        db: AdminBotSecurityContentVerificationLlmConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::ContentModerationLlmFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;
        let llm = file.llm.unwrap_or_default().merge_with(base_llm)?;

        Some(Self { db, llm })
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
            llm: FaceVerificationLlmConfigInternal::new(db.llm, db.llm_enabled, file.llm, base_llm),
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
            llm_primary: ContentModerationLlmConfigInternal::new(
                db.llm_primary,
                db.llm_primary_enabled,
                file.llm_primary,
                base_llm.clone(),
            ),
            llm_secondary: ContentModerationLlmConfigInternal::new(
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
pub struct ContentModerationLlmConfigInternal {
    pub db: AdminBotContentModerationLlmConfig,
    pub llm: LlmConfig,
}

#[derive(Debug, Clone)]
pub struct FaceVerificationLlmConfigInternal {
    pub db: AdminBotFaceVerificationLlmConfig,
    pub llm: LlmConfig,
}

impl FaceVerificationLlmConfigInternal {
    pub fn new(
        db: AdminBotFaceVerificationLlmConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::ContentModerationLlmFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;
        let llm = file.llm.unwrap_or_default().merge_with(base_llm)?;

        Some(Self { db, llm })
    }
}

impl ContentModerationLlmConfigInternal {
    pub fn new(
        db: AdminBotContentModerationLlmConfig,
        db_enabled: bool,
        file: Option<crate::bot_config_file::ContentModerationLlmFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<Self> {
        if !db_enabled {
            return None;
        }
        let file = file?;
        let llm = file.llm.unwrap_or_default().merge_with(base_llm)?;

        Some(Self { db, llm })
    }
}

#[derive(Debug, Clone)]
pub struct ReportProcessingProfileStringConfigInternal {
    pub db: AdminBotReportProcessingProfileStringLlmConfig,
    pub llm: LlmConfig,
    pub default_action: AcceptOrReject,
}

impl ReportProcessingProfileStringConfigInternal {
    pub const TEMPLATE_PLACEHOLDER_TEXT: &str = TEMPLATE_PLACEHOLDER_TEXT;

    pub fn new(
        db: AdminBotReportProcessingProfileStringLlmConfig,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
        default_action: AcceptOrReject,
    ) -> Option<Self> {
        let file = file?;
        let llm = file.llm.unwrap_or_default().merge_with(base_llm)?;
        Some(Self {
            db,
            llm,
            default_action,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ReportProcessingProfileContentConfigInternal {
    pub db: AdminBotReportProcessingProfileContentLlmConfig,
    pub llm: LlmConfig,
    pub default_action: AcceptOrReject,
}

impl ReportProcessingProfileContentConfigInternal {
    pub fn new(
        db: AdminBotReportProcessingProfileContentLlmConfig,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
        default_action: AcceptOrReject,
    ) -> Option<Self> {
        let file = file?;
        let llm = file.llm.unwrap_or_default().merge_with(base_llm)?;
        Some(Self {
            db,
            llm,
            default_action,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ReportProcessingMessagesConfigInternal {
    pub db: AdminBotReportProcessingMessagesLlmConfig,
    pub llm: LlmConfig,
    pub default_action: AcceptOrReject,
}

impl ReportProcessingMessagesConfigInternal {
    pub const TEMPLATE_PLACEHOLDER_TEXT: &str = TEMPLATE_PLACEHOLDER_TEXT;

    pub fn new(
        db: AdminBotReportProcessingMessagesLlmConfig,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
        default_action: AcceptOrReject,
    ) -> Option<Self> {
        let file = file?;
        let llm = file.llm.unwrap_or_default().merge_with(base_llm)?;
        Some(Self {
            db,
            llm,
            default_action,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ReportProcessingConfigInternal {
    pub profile_name: Option<ReportProcessingProfileStringConfigInternal>,
    pub profile_text: Option<ReportProcessingProfileStringConfigInternal>,
    pub profile_content: Option<ReportProcessingProfileContentConfigInternal>,
    pub messages: Option<ReportProcessingMessagesConfigInternal>,
    pub concurrency: u8,
}

impl ReportProcessingConfigInternal {
    fn new_per_type_profile_string(
        db_llm: AdminBotReportProcessingProfileStringLlmConfig,
        db_enabled: bool,
        default_action: AcceptOrReject,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<ReportProcessingProfileStringConfigInternal> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        ReportProcessingProfileStringConfigInternal::new(
            db_llm,
            Some(file),
            base_llm,
            default_action,
        )
    }

    fn new_per_type_profile_content(
        db_llm: AdminBotReportProcessingProfileContentLlmConfig,
        db_enabled: bool,
        default_action: AcceptOrReject,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<ReportProcessingProfileContentConfigInternal> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        ReportProcessingProfileContentConfigInternal::new(
            db_llm,
            Some(file),
            base_llm,
            default_action,
        )
    }

    fn new_per_type_messages(
        db_llm: AdminBotReportProcessingMessagesLlmConfig,
        db_enabled: bool,
        default_action: AcceptOrReject,
        file: Option<crate::bot_config_file::ReportProcessingTypeFileConfig>,
        base_llm: crate::bot_config_file::BaseLlmConfig,
    ) -> Option<ReportProcessingMessagesConfigInternal> {
        if !db_enabled {
            return None;
        }
        let file = file?;

        ReportProcessingMessagesConfigInternal::new(db_llm, Some(file), base_llm, default_action)
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
