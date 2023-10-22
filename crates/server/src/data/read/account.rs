use error_stack::{FutureExt, Result, ResultExt};
use model::{
    AccessToken, Account, AccountId, AccountIdInternal, AccountSetup, GoogleAccountId,
    RefreshToken, SignInWithInfo, AccountData,
};
use tokio_stream::StreamExt;

use super::{
    super::{cache::DatabaseCache, file::utils::FileDir, DataError},
    ReadCommands,
};
use crate::data::IntoDataError;

define_read_commands!(ReadCommandsAccount);

impl ReadCommandsAccount<'_> {
    pub async fn account_access_token(
        &self,
        id: AccountId,
    ) -> Result<Option<AccessToken>, DataError> {
        let id = self
            .cache()
            .to_account_id_internal(id)
            .await
            .into_data_error(id)?;
        self.db_read(move |mut cmds| cmds.account().access_token(id))
            .await
    }

    pub async fn account_refresh_token(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<RefreshToken>, DataError> {
        self.db_read(move |mut cmds| cmds.account().refresh_token(id))
            .await
    }

    pub async fn account_sign_in_with_info(
        &self,
        id: AccountIdInternal,
    ) -> Result<SignInWithInfo, DataError> {
        self.db_read(move |mut cmds| cmds.account().sign_in_with_info(id))
            .await
    }

    pub async fn account(&self, id: AccountIdInternal) -> Result<Account, DataError> {
        self.read_cache(id, |cache| cache.account.as_deref().map(Clone::clone))
            .await?
            .ok_or(DataError::Cache.report())
    }

    pub async fn account_data(&self, id: AccountIdInternal) -> Result<AccountData, DataError> {
        self.db_read(move |mut cmds| cmds.account().account_data(id))
            .await
    }

    pub async fn account_setup(&self, id: AccountIdInternal) -> Result<AccountSetup, DataError> {
        self.db_read(move |mut cmds| cmds.account().account_setup(id))
            .await
    }

    pub async fn account_ids<T: FnMut(AccountIdInternal)>(
        &self,
        mut handler: T,
    ) -> Result<(), DataError> {
        let account = self.db().account();
        let mut users = account.account_ids_stream();
        while let Some(user_id) = users.try_next().await.change_context(DataError::Sqlite)? {
            handler(user_id)
        }

        Ok(())
    }

    pub async fn google_account_id_to_account_id(
        &self,
        id: GoogleAccountId,
    ) -> Result<Option<AccountIdInternal>, DataError> {
        self.db_read(move |mut cmds| cmds.account().google_account_id_to_account_id(id))
            .await
            .map(Some)
    }
}
