use error_stack::{FutureExt, Result, ResultExt};
use model::{
    AccessToken, Account, AccountId, AccountIdInternal, AccountSetup, GoogleAccountId,
    RefreshToken, SignInWithInfo, EventToClient, SharedStateInternal, SharedState,
};
use tokio_stream::StreamExt;

use super::{
    super::{cache::DatabaseCache, file::utils::FileDir, DataError},
    ReadCommands,
};
use crate::{data::IntoDataError, event::{EventSender, EventMode}};

define_read_commands!(ReadCommandsCommon);

impl ReadCommandsCommon<'_> {
    pub async fn access_event_mode<T>(
        &self,
        id: AccountId,
        action: impl FnOnce(&EventMode) -> T,
    ) -> Result<T, DataError> {
        self
            .cache()
            .read_cache(id, move |entry| {
                action(&entry.current_event_connection)
            })
            .await
            .into_data_error(id)
    }

    pub async fn shared_state(&self, id: AccountIdInternal) -> Result<SharedState, DataError> {
        self.db_read(move |mut cmds| cmds.common().shared_state(id))
            .await
    }

    // pub async fn <T>(
    //     &self,
    //     id: AccountId,
    // ) -> Result<SharedState, DataError> {
    //     self
    //         .cache()
    //         .read_cache(id, move |entry| {
    //             action(&entry.current_event_connection)
    //         })
    //         .await
    //         .into_data_error(id)
    // }
}
