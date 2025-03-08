use chrono::NaiveDate;
use database::current::read::GetDbReadCommandsCommon;
use model::{
    AccessToken, Account, AccountId, AccountIdInternal, PendingNotificationFlags, RefreshToken,
};
use server_common::data::IntoDataError;

use super::{super::DataError, DbRead};
use crate::{
    cache::CacheReadCommon, db_manager::InternalReading, define_cmd_wrapper_read, id::ToAccountIdInternal, result::Result
};

define_cmd_wrapper_read!(ReadCommandsCommon);

impl ReadCommandsCommon<'_> {
    pub async fn account_access_token(
        &self,
        id: AccountId,
    ) -> Result<Option<AccessToken>, DataError> {
        let id = self.to_account_id_internal(id).await.into_data_error(id)?;
        self.db_read(move |mut cmds| cmds.common().token().access_token(id))
            .await
            .into_error()
    }

    pub async fn account_refresh_token(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<RefreshToken>, DataError> {
        self.db_read(move |mut cmds| cmds.common().token().refresh_token(id))
            .await
            .into_error()
    }

    /// Account is available on all servers as account server will sync it to
    /// others if server is running in microservice mode.
    pub async fn account(&self, id: AccountIdInternal) -> Result<Account, DataError> {
        let account = self
            .read_cache_common(id, |cache| {
                Ok(Account::new_from_internal_types(
                    cache.permissions.clone(),
                    cache.account_state_related_shared_state.clone(),
                ))
            })
            .await?;
        Ok(account)
    }

    pub async fn cached_pending_notification_flags(
        &self,
        id: AccountIdInternal,
    ) -> Result<PendingNotificationFlags, DataError> {
        let flags = self
            .read_cache_common(id, |cache| Ok(cache.pending_notification_flags))
            .await?;
        Ok(flags)
    }

    pub async fn latest_birthdate(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<NaiveDate>, DataError> {
        self.db_read(move |mut cmds| cmds.common().state().other_shared_state(id))
            .await
            .into_error()
            .map(|v| v.birthdate)
    }

    pub async fn account_ids_vec(&self) -> Result<Vec<AccountId>, DataError> {
        self.db_read(move |mut cmds| cmds.common().account_ids())
            .await
            .into_error()
    }

    pub async fn account_ids_internal_vec(&self) -> Result<Vec<AccountIdInternal>, DataError> {
        self.db_read(move |mut cmds| cmds.common().account_ids_internal())
            .await
            .into_error()
    }

    pub async fn backup_current_database(&self, file_name: String) -> Result<(), DataError> {
        self.db_read_raw_no_transaction(move |mut cmds| cmds.common().backup_current_database(file_name))
            .await
            .into_error()
    }
}
