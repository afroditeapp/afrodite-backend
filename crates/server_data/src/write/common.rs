use std::net::SocketAddr;

use model::{Account, AccountId, AccountIdInternal, AuthPair};
use server_common::data::cache::CacheError;

use crate::{
    event::{event_channel, EventMode, EventReceiver},
    result::Result,
    write::db_transaction,
    DataError, IntoDataError,
};

use super::WriteCommandsProvider;

define_write_commands!(WriteCommandsCommon);

impl <C: WriteCommandsProvider> WriteCommandsCommon<C> {
    pub async fn set_new_auth_pair(
        &self,
        id: AccountIdInternal,
        pair: AuthPair,
        address: Option<SocketAddr>,
    ) -> Result<(), DataError> {
        let access = pair.access.clone();
        let current_access_token = db_transaction!(self, move |mut cmds| {
            let current_access_token = cmds.read().common().token().access_token(id)?;
            cmds.common().token().access_token(id, Some(access))?;
            cmds.common()
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
            let current_access_token = cmds.read().common().token().access_token(id);
            cmds.common().token().access_token(id, None)?;
            cmds.common().token().refresh_token(id, None)?;
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

    pub async fn internal_handle_new_account_data_after_db_modification(
        &self,
        id: AccountIdInternal,
        current_account: &Account,
        new_account: &Account,
    ) -> Result<(), DataError> {
        let new_account_clone = new_account.clone();
        self.write_cache(id, |cache| {
            cache.capabilities = new_account_clone.capablities();
            cache.shared_state = new_account_clone.into();
            Ok(())
        })
        .await?;

        // Other related state updating

        if self.config().components().profile
            && current_account.profile_visibility().is_currently_public()
                != new_account.profile_visibility().is_currently_public()
        {
            self.profile_update_location_index_visibility(
                id,
                new_account.profile_visibility().is_currently_public(),
            )
            .await?;
        }

        Ok(())
    }

    pub async fn profile_update_location_index_visibility(
        &self,
        id: AccountIdInternal,
        visibility: bool,
    ) -> Result<(), DataError> {
        let (location, profile_data) = self
            .cache()
            .write_cache(id.as_id(), |e| {
                let p = e.profile.as_mut().ok_or(CacheError::FeatureNotEnabled)?;

                Ok((p.location.current_position, p.location_index_profile_data()))
            })
            .await
            .into_data_error(id)?;

        if visibility {
            self.location()
                .update_profile_data(id.as_id(), profile_data, location)
                .await?;
        } else {
            self.location()
                .remove_profile_data(id.as_id(), location)
                .await?;
        }

        Ok(())
    }
}
