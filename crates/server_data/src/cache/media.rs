use model::{AccountId, ProfileContentVersion};
use model_server_data::ProfileContentEditedTime;

#[derive(Debug)]
pub struct CachedMedia {
    pub account_id: AccountId,
    pub profile_content_version: ProfileContentVersion,
    pub profile_content_edited_time: ProfileContentEditedTime,
}

impl CachedMedia {
    pub fn new(
        account_id: AccountId,
        profile_content_version: ProfileContentVersion,
        profile_content_edited_time: ProfileContentEditedTime,
    ) -> Self {
        Self {
            account_id,
            profile_content_version,
            profile_content_edited_time,
        }
    }
}
