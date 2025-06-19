use model_profile::AccountIdInternal;
use server_data::{DataError, IntoDataError, define_cmd_wrapper_read, result::Result};

use crate::cache::CacheReadProfile;

define_cmd_wrapper_read!(ReadCommandsProfileSearch);

impl ReadCommandsProfileSearch<'_> {
    pub async fn automatic_profile_search_happened_at_least_once(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<bool, DataError> {
        self.read_cache_profile_and_common(account_id, |p, _| {
            Ok(p.automatic_profile_search.last_seen_unix_time.is_some())
        })
        .await
        .into_error()
    }
}
