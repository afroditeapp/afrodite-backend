use database_profile::current::write::GetDbWriteCommandsProfile;
use model_profile::{AccountIdInternal, ProfilePrivacySettings};
use server_data::{
    DataError, IntoDataError, db_transaction, define_cmd_wrapper_write, result::Result,
    write::DbTransaction,
};

use crate::cache::CacheWriteProfile;

define_cmd_wrapper_write!(WriteCommandsProfilePrivacy);

impl WriteCommandsProfilePrivacy<'_> {
    pub async fn upsert_privacy_settings(
        &self,
        id: AccountIdInternal,
        value: ProfilePrivacySettings,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile().privacy().upsert_privacy_settings(id, value)
        })?;

        self.write_cache_profile(id.as_id(), |p| {
            p.update_privacy_settings(value);
            Ok(())
        })
        .await
        .into_data_error(id)?;

        Ok(())
    }
}
