use model_profile::AccountIdInternal;
use server_data::{DataError, IntoDataError, define_cmd_wrapper_write, result::Result};

use crate::cache::CacheWriteProfile;

define_cmd_wrapper_write!(WriteCommandsProfileAdminNotification);

impl WriteCommandsProfileAdminNotification<'_> {
    pub async fn show_automatic_profile_search_notification(
        &self,
        id: AccountIdInternal,
        profile_count: i64,
    ) -> Result<(), DataError> {
        self.write_cache_profile(id, |p| {
            p.automatic_profile_search.notification.profiles_found.id = p
                .automatic_profile_search
                .notification
                .profiles_found
                .id
                .wrapping_increment();
            p.automatic_profile_search.notification.profile_count = profile_count;
            Ok(())
        })
        .await
        .into_error()
    }
}
