use database::NoId;
use tokio_stream::StreamExt;

use model::{AccountIdInternal, AccountIdLight, ApiKey, RefreshToken, SignInWithInfo};

use crate::utils::ConvertCommandErrorExt;

use super::{
    super::{cache::DatabaseCache, file::utils::FileDir, DatabaseError},
    ReadCommands,
};

use error_stack::Result;

define_read_commands!(ReadCommandsAccount);

impl ReadCommandsAccount<'_> {
    pub async fn account_access_token(
        &self,
        id: AccountIdLight,
    ) -> Result<Option<ApiKey>, DatabaseError> {
        let id = self.cache().to_account_id_internal(id).await.convert(id)?;
        self.db().account().access_token(id).await.convert(id)
    }

    pub async fn account_refresh_token(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<RefreshToken>, DatabaseError> {
        self.db().account().refresh_token(id).await.convert(id)
    }

    pub async fn account_sign_in_with_info(
        &self,
        id: AccountIdInternal,
    ) -> Result<SignInWithInfo, DatabaseError> {
        self.db().account().sign_in_with_info(id).await.convert(id)
    }

    pub async fn account_ids<T: FnMut(AccountIdInternal)>(
        &self,
        mut handler: T,
    ) -> Result<(), DatabaseError> {
        let account = self.db().account();
        let mut users = account.account_ids_stream();
        while let Some(user_id) = users.try_next().await.convert(NoId)? {
            handler(user_id)
        }

        Ok(())
    }
}
