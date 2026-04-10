use model::{AccountId, MediaVerificationStatusFlags, ProfileContentVersion};
use model_server_data::ProfileContentEditedTime;

#[derive(Debug)]
pub struct CacheMedia {
    pub account_id: AccountId,
    pub profile_content_version: ProfileContentVersion,
    pub profile_content_edited_time: ProfileContentEditedTime,
    pub media_verification_status_flags: MediaVerificationStatusFlags,
}

impl CacheMedia {
    pub fn new(
        account_id: AccountId,
        profile_content_version: ProfileContentVersion,
        profile_content_edited_time: ProfileContentEditedTime,
        media_verification_status_flags: MediaVerificationStatusFlags,
    ) -> Self {
        Self {
            account_id,
            profile_content_version,
            profile_content_edited_time,
            media_verification_status_flags,
        }
    }
}
