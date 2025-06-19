use database_profile::current::write::GetDbWriteCommandsProfile;
use model_profile::AccountIdInternal;
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

use crate::cache::CacheWriteProfile;

define_cmd_wrapper_write!(WriteCommandsProfileAdminNotification);

impl WriteCommandsProfileAdminNotification<'_> {
    pub async fn show_profile_text_accepted_notification(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile_admin()
                .notification()
                .show_profile_text_accepted_notification(id)?;
            Ok(())
        })?;

        Ok(())
    }

    pub async fn show_profile_text_rejected_notification(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile_admin()
                .notification()
                .show_profile_text_rejected_notification(id)?;
            Ok(())
        })?;

        Ok(())
    }

    pub async fn show_automatic_profile_search_notification(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        self.write_cache_profile(id, |p| {
            p.automatic_profile_search.notification.profiles_found = p
                .automatic_profile_search
                .notification
                .profiles_found
                .wrapping_add(1);
            Ok(())
        })
        .await
        .into_error()
    }
}
