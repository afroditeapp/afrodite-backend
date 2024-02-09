use model::{
    AccessToken, Account, AccountData, AccountId, AccountIdInternal, AccountSetup, GoogleAccountId,
    RefreshToken, SignInWithInfo,
};
use tokio_stream::StreamExt;

use super::{
    super::{cache::DatabaseCache, file::utils::FileDir, DataError},
    ReadCommands,
};
use crate::{data::IntoDataError, result::Result};

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
        self.db_read(move |mut cmds| cmds.account().token().access_token(id))
            .await
            .into_error()
    }

    pub async fn account_refresh_token(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<RefreshToken>, DataError> {
        self.db_read(move |mut cmds| cmds.account().token().refresh_token(id))
            .await
            .into_error()
    }

    pub async fn account_sign_in_with_info(
        &self,
        id: AccountIdInternal,
    ) -> Result<SignInWithInfo, DataError> {
        self.db_read(move |mut cmds| cmds.account().sign_in_with().sign_in_with_info(id))
            .await
            .into_error()
    }

    pub async fn account(&self, id: AccountIdInternal) -> Result<Account, DataError> {
        let account = self
            .read_cache(id, |cache| {
                Account::new_from(cache.shared_state.account_state, cache.capabilities.clone())
            })
            .await?;
        Ok(account)
    }

    pub async fn account_data(&self, id: AccountIdInternal) -> Result<AccountData, DataError> {
        self.db_read(move |mut cmds| cmds.account().data().account_data(id))
            .await
            .into_error()
    }

    pub async fn account_setup(&self, id: AccountIdInternal) -> Result<AccountSetup, DataError> {
        self.db_read(move |mut cmds| cmds.account().data().account_setup(id))
            .await
            .into_error()
    }

    pub async fn account_ids<T: FnMut(AccountIdInternal)>(
        &self,
        mut handler: T,
    ) -> Result<(), DataError> {
        let db = self.db();
        let account = db.account();
        let data = account.data();
        let mut users = data.account_ids_stream();
        while let Some(user_id) = users.try_next().await? {
            handler(user_id)
        }

        Ok(())
    }

    pub async fn google_account_id_to_account_id(
        &self,
        id: GoogleAccountId,
    ) -> Result<Option<AccountIdInternal>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account()
                .sign_in_with()
                .google_account_id_to_account_id(id)
        })
        .await
        .map(Some)
        .into_error()
    }
}
