use config::bot_config_file::internal::{
    AccountVerificationConfigInternal, ContentModerationConfigInternal,
    FaceVerificationConfigInternal, ProfileStringModerationConfigInternal,
    ReportProcessingConfigInternal,
};
use tracing::warn;

fn warn_missing_llm_config(name: &str) {
    warn!(
        "Admin bot {} is enabled but LLM URL and model not configured properly in bot config file",
        name,
    );
}

pub fn log_warnings(
    profile_name: &Option<ProfileStringModerationConfigInternal>,
    profile_text: &Option<ProfileStringModerationConfigInternal>,
    content: &Option<ContentModerationConfigInternal>,
    face_verification: &Option<FaceVerificationConfigInternal>,
    account_verification: &Option<AccountVerificationConfigInternal>,
    report_processing: &Option<ReportProcessingConfigInternal>,
) {
    if let Some(c) = profile_name
        && c.llm.is_none()
    {
        warn_missing_llm_config("profile name moderation");
    }

    if let Some(c) = profile_text
        && c.llm.is_none()
    {
        warn_missing_llm_config("profile text moderation");
    }

    if let Some(c) = content {
        if c.llm_primary.is_none() {
            warn_missing_llm_config("content moderation (primary LLM)");
        }
        if c.llm_secondary.is_none() {
            warn_missing_llm_config("content moderation (secondary LLM)");
        }
    }

    if let Some(c) = face_verification
        && c.llm.is_none()
    {
        warn_missing_llm_config("face verification");
    }

    if let Some(c) = account_verification
        && let Some(s) = &c.security_content
        && s.llm.is_none()
    {
        warn_missing_llm_config("security content verification");
    }

    if let Some(c) = report_processing {
        if c.profile_name.is_none() {
            warn_missing_llm_config("report processing (profile name)");
        }
        if c.profile_text.is_none() {
            warn_missing_llm_config("report processing (profile text)");
        }
        if c.profile_content.is_none() {
            warn_missing_llm_config("report processing (profile content)");
        }
        if c.messages.is_none() {
            warn_missing_llm_config("report processing (messages)");
        }
    }
}
