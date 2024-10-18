use model::{AccountId, ProfileContentVersion};

#[derive(Debug)]
pub struct CachedMedia {
    pub account_id: AccountId,
    pub profile_content_version: ProfileContentVersion,
}

impl CachedMedia {
    pub fn new(
        account_id: AccountId,
        profile_content_version: ProfileContentVersion
    ) -> Self {
        Self {
            account_id,
            profile_content_version,
        }
    }
}
