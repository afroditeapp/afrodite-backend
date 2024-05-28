use model::{Account, AccountId, AccountIdInternal};

use super::super::DataError;
use crate::{data::IntoDataError, event::EventMode, result::Result};

define_read_commands!(ReadCommandsCommon);

impl ReadCommandsCommon<'_> {
    pub async fn access_event_mode<T>(
        &self,
        id: AccountId,
        action: impl FnOnce(&EventMode) -> T,
    ) -> Result<T, DataError> {
        self.cache()
            .read_cache(id, move |entry| action(&entry.current_event_connection))
            .await
            .into_data_error(id)
    }

    /// Account is available on all servers as account server will sync it to
    /// others if server is running in microservice mode.
    pub async fn account(&self, id: AccountIdInternal) -> Result<Account, DataError> {
        let account = self
            .read_cache(id, |cache| {
                Account::new_from_internal_types(
                    cache.capabilities.clone(),
                    cache.shared_state.clone(),
                )
            })
            .await?;
        Ok(account)
    }
}
