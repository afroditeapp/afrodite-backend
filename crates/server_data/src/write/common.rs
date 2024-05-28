use std::net::SocketAddr;

use model::{AccountId, AccountIdInternal, AuthPair};

use crate::{
    event::{event_channel, EventMode, EventReceiver},
    result::Result,
    write::db_transaction,
    DataError, IntoDataError,
};

define_write_commands!(WriteCommandsCommon);

impl WriteCommandsCommon<'_> {
    pub async fn set_new_auth_pair(
        &self,
        id: AccountIdInternal,
        pair: AuthPair,
        address: Option<SocketAddr>,
    ) -> Result<(), DataError> {
        let access = pair.access.clone();
        let current_access_token = db_transaction!(self, move |mut cmds| {
            let current_access_token = cmds.read().account().token().access_token(id)?;
            cmds.account().token().access_token(id, Some(access))?;
            cmds.account()
                .token()
                .refresh_token(id, Some(pair.refresh))?;
            Ok(current_access_token)
        })?;

        self.cache()
            .update_access_token_and_connection(
                id.as_id(),
                current_access_token,
                pair.access,
                address,
            )
            .await
            .into_data_error(id)
    }

    /// Remove current connection address, access and refresh tokens.
    pub async fn logout(&self, id: AccountIdInternal) -> Result<(), DataError> {
        let current_access_token = db_transaction!(self, move |mut cmds| {
            let current_access_token = cmds.read().account().token().access_token(id);
            cmds.account().token().access_token(id, None)?;
            cmds.account().token().refresh_token(id, None)?;
            current_access_token
        })?;

        self.cache()
            .delete_connection_and_specific_access_token(id.as_id(), current_access_token)
            .await
            .into_data_error(id)?;

        Ok(())
    }

    /// Init event channel for connection session.
    pub async fn init_connection_session_events(
        &self,
        id: AccountId,
    ) -> Result<EventReceiver, DataError> {
        let (sender, receiver) = event_channel();
        self.write_cache(id, move |entry| {
            entry.current_event_connection = EventMode::Connected(sender);
            Ok(())
        })
        .await?;

        Ok(receiver)
    }

    // TODO(prod): Logout route which removes current
    //             tokens and connection address.

    /// Remove current connection address and access token.
    pub async fn end_connection_session(&self, id: AccountIdInternal) -> Result<(), DataError> {
        self.cache()
            .delete_connection_and_specific_access_token(id.as_id(), None)
            .await
            .into_data_error(id)?;

        Ok(())
    }
}
