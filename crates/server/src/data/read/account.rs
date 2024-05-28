use model::{
    AccessToken, Account, AccountData, AccountGlobalState, AccountId, AccountIdInternal, AccountSetup, DemoModeId, GoogleAccountId, RefreshToken, SignInWithInfo
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
        self.db_read(move |mut cmds| cmds.account().sign_in_with().sign_in_with_info_raw(id).map(|v| v.into()))
            .await
            .into_error()
    }

    pub async fn is_bot_account(
        &self,
        id: AccountIdInternal,
    ) -> Result<bool, DataError> {
        self.db_read(move |mut cmds| cmds.account().sign_in_with().sign_in_with_info_raw(id).map(|v| v.is_bot_account))
            .await
            .into_error()
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

    pub async fn account_ids_vec(
        &self,
    ) -> Result<Vec<AccountId>, DataError> {
        self.db_read(move |mut cmds| cmds.account().data().account_ids())
            .await
            .into_error()
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
        .into_error()
    }

    pub async fn demo_mode_related_account_ids(
        &self,
        id: DemoModeId,
    ) -> Result<Vec<AccountId>, DataError> {
        self.db_read(move |mut cmds| cmds.account().demo_mode().related_account_ids(id))
            .await
            .into_error()
    }

    pub async fn global_state(
        &self,
    ) -> Result<AccountGlobalState, DataError> {
        self.db_read(move |mut cmds| cmds.account().data().global_state())
            .await
            .into_error()
    }
}
