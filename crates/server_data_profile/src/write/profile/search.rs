use database_profile::current::write::GetDbWriteCommandsProfile;
use model_profile::{AccountIdInternal, AutomaticProfileSearchSettings};
use server_data::{
    DataError, IntoDataError, db_transaction, define_cmd_wrapper_write, result::Result,
    write::DbTransaction,
};

use crate::cache::CacheWriteProfile;

define_cmd_wrapper_write!(WriteCommandsProfileSearch);

impl WriteCommandsProfileSearch<'_> {
    pub async fn upsert_automatic_profile_search_settings(
        &self,
        id: AccountIdInternal,
        value: AutomaticProfileSearchSettings,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .search()
                .upsert_automatic_profile_search_settings(id, value)
        })?;

        self.write_cache_profile(id, |entry| {
            entry.automatic_profile_search.update_settings(value);
            Ok(())
        })
        .await
        .into_error()
    }
}
