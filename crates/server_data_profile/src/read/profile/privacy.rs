use model_profile::{AccountIdInternal, ProfilePrivacySettings};
use server_data::{DataError, IntoDataError, define_cmd_wrapper_read, result::Result};

use crate::cache::CacheReadProfile;

define_cmd_wrapper_read!(ReadCommandsProfilePrivacy);

impl ReadCommandsProfilePrivacy<'_> {
    pub async fn profile_privacy_settings(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfilePrivacySettings, DataError> {
        self.read_cache_profile_and_common(id.as_id(), |p, _| Ok(p.privacy_settings()))
            .await
            .into_error()
    }
}
