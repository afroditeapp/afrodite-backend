use model::{AccessToken, Account, AccountId, AccountIdInternal, PendingNotificationFlags, RefreshToken};

use super::{super::DataError, ReadCommandsProvider};
use crate::{result::Result, IntoDataError};

define_read_commands!(ReadCommandsCommon);

impl<C: ReadCommandsProvider> ReadCommandsCommon<C> {
    pub async fn account_access_token(
        &self,
        id: AccountId,
    ) -> Result<Option<AccessToken>, DataError> {
        let id = self
            .cache()
            .to_account_id_internal(id)
            .await
            .into_data_error(id)?;
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
            .read_cache(id, |cache| {
                Account::new_from_internal_types(
                    cache.capabilities.clone(),
                    cache.account_state_related_shared_state.clone(),
                )
            })
            .await?;
        Ok(account)
    }

    pub async fn cached_pending_notification_flags(
        &self,
        id: AccountIdInternal,
    ) -> Result<PendingNotificationFlags, DataError> {
        let flags = self.read_cache(id, |cache| cache.pending_notification_flags)
            .await?;
        Ok(flags)
    }
}
