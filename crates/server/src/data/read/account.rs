use error_stack::{Result, FutureExt, ResultExt};
use tokio_stream::StreamExt;


use model::{AccountIdInternal, AccountIdLight, ApiKey, RefreshToken, SignInWithInfo, GoogleAccountId, Account, AccountSetup};

use crate::utils::{ErrorConversion};

use super::{
    ReadCommands,
    super::{cache::DatabaseCache, DatabaseError, file::utils::FileDir},
};

define_read_commands!(ReadCommandsAccount);

impl ReadCommandsAccount<'_> {
    pub async fn account_access_token(
        &self,
        id: AccountIdLight,
    ) -> Result<Option<ApiKey>, DatabaseError> {
        let id = self.cache().to_account_id_internal(id).await.with_info(id)?;
        self.db_read(move |cmds| cmds.account().access_token(id))
            .await
    }

    pub async fn account_refresh_token(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<RefreshToken>, DatabaseError> {
        self.db_read(move |cmds| cmds.account().refresh_token(id))
            .await
    }

    pub async fn account_sign_in_with_info(
        &self,
        id: AccountIdInternal,
    ) -> Result<SignInWithInfo, DatabaseError> {
        self.db_read(move |cmds| cmds.account().sign_in_with_info(id)).await
    }

    pub async fn account(
        &self,
        id: AccountIdInternal,
    ) -> Result<Account, DatabaseError> {
        self.read_cache(id, |cache| cache.account.as_deref().map(Clone::clone)).await?
            .ok_or(DatabaseError::Cache.into())
    }

    pub async fn account_setup(
        &self,
        id: AccountIdInternal,
    ) -> Result<AccountSetup, DatabaseError> {
        self.db_read(move |cmds| cmds.account().account_setup(id)).await
    }

    pub async fn account_ids<T: FnMut(AccountIdInternal)>(
        &self,
        mut handler: T,
    ) -> Result<(), DatabaseError> {
        let account = self.db().account();
        let mut users = account.account_ids_stream();
        while let Some(user_id) = users.try_next().await.change_context(DatabaseError::Sqlite)? {
            handler(user_id)
        }

        Ok(())
    }

    pub async fn google_account_id_to_account_id(
        &self,
        id: GoogleAccountId,
    ) -> Result<Option<AccountIdInternal>, DatabaseError> {
        self.db_read(move |cmds| cmds.account().google_account_id_to_account_id(id))
            .await
            .map(Some)
    }
}
