use config::{
    AdminBotConfig,
    bot_config_file::internal::{
        AccountVerificationConfigInternal, ContentModerationConfigInternal,
        FaceVerificationConfigInternal, ProfileStringModerationConfigInternal,
        ReportProcessingConfigInternal,
    },
};
use tracing::warn;

fn warn_missing_llm_config(name: &str) {
    warn!(
        "Admin bot {} is enabled but LLM URL and model not configured properly in bot config file",
        name,
    );
}

pub fn log_warnings(
    db: &AdminBotConfig,
    profile_name: &Option<ProfileStringModerationConfigInternal>,
    profile_text: &Option<ProfileStringModerationConfigInternal>,
    content: &Option<ContentModerationConfigInternal>,
    face_verification: &Option<FaceVerificationConfigInternal>,
    account_verification: &Option<AccountVerificationConfigInternal>,
    report_processing: &Option<ReportProcessingConfigInternal>,
) {
    if let Some(c) = profile_name
        && c.llm.is_none()
        && db.profile_name_moderation.llm_enabled
    {
        warn_missing_llm_config("profile name moderation");
    }

    if let Some(c) = profile_text
        && c.llm.is_none()
        && db.profile_text_moderation.llm_enabled
    {
        warn_missing_llm_config("profile text moderation");
    }

    if let Some(c) = content {
        if c.llm_primary.is_none() && db.content_moderation.llm_primary_enabled {
            warn_missing_llm_config("content moderation (primary LLM)");
        }
        if c.llm_secondary.is_none() && db.content_moderation.llm_secondary_enabled {
            warn_missing_llm_config("content moderation (secondary LLM)");
        }
    }

    if let Some(c) = face_verification
        && c.llm.is_none()
        && db.face_verification.llm_enabled
    {
        warn_missing_llm_config("face verification");
    }

    if let Some(c) = account_verification
        && let Some(s) = &c.security_content
        && s.llm.is_none()
        && db.account_verification.security_content.llm_enabled
    {
        warn_missing_llm_config("security content verification");
    }

    if let Some(c) = report_processing {
        if c.profile_name.is_none() && db.report_processing.profile_name.llm_enabled {
            warn_missing_llm_config("report processing (profile name)");
        }
        if c.profile_text.is_none() && db.report_processing.profile_text.llm_enabled {
            warn_missing_llm_config("report processing (profile text)");
        }
        if c.profile_content.is_none() && db.report_processing.profile_content.llm_enabled {
            warn_missing_llm_config("report processing (profile content)");
        }
        if c.messages.is_none() && db.report_processing.messages.llm_enabled {
            warn_missing_llm_config("report processing (messages)");
        }
    }
}
