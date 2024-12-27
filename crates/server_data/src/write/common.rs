use std::net::SocketAddr;

use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use model::{Account, AccountId, AccountIdInternal};
use model_server_data::AuthPair;
use server_common::data::cache::CacheError;

use super::DbTransaction;
use crate::{
    cache::{CacheWriteCommon, LastSeenTimeUpdated, TopLevelCacheOperations},
    db_manager::InternalWriting,
    define_cmd_wrapper_write,
    event::EventReceiver,
    file::FileWrite,
    result::Result,
    write::db_transaction,
    DataError, IntoDataError,
};

define_cmd_wrapper_write!(WriteCommandsCommon);

impl WriteCommandsCommon<'_> {
    /// Creates new event channel if address is Some.
    pub async fn set_new_auth_pair(
        &self,
        id: AccountIdInternal,
        pair: AuthPair,
        address: Option<SocketAddr>,
    ) -> Result<Option<EventReceiver>, DataError> {
        let access = pair.access.clone();
        let current_access_token = db_transaction!(self, move |mut cmds| {
            let current_access_token = cmds.read().common().token().access_token(id)?;
            cmds.common().token().access_token(id, Some(access))?;
            cmds.common()
                .token()
                .refresh_token(id, Some(pair.refresh))?;
            Ok(current_access_token)
        })?;

        let option = self
            .update_access_token_and_connection(
                id.as_id(),
                current_access_token,
                pair.access,
                address,
            )
            .await
            .into_data_error(id)?;

        if let Some(last_seen_time_update) = option.as_ref().and_then(|v| v.1) {
            self.update_last_seen_time(id.uuid, last_seen_time_update)
                .await;
        }

        Ok(option.map(|v| v.0))
    }

    /// Remove current connection address, access and refresh tokens.
    pub async fn logout(&self, id: AccountIdInternal) -> Result<(), DataError> {
        let current_access_token = db_transaction!(self, move |mut cmds| {
            let current_access_token = cmds.read().common().token().access_token(id);
            cmds.common().token().access_token(id, None)?;
            cmds.common().token().refresh_token(id, None)?;
            current_access_token
        })?;

        let last_seen_time_update = self
            .delete_connection_and_specific_access_token(id.as_id(), None, current_access_token)
            .await
            .into_data_error(id)?;

        if let Some(last_seen_time_update) = last_seen_time_update {
            self.update_last_seen_time(id.uuid, last_seen_time_update)
                .await;
        }

        Ok(())
    }

    /// Remove specific connection session.
    pub async fn end_connection_session(
        &self,
        id: AccountIdInternal,
        session_address: SocketAddr,
    ) -> Result<(), DataError> {
        let last_seen_time_update = self
            .delete_connection_and_specific_access_token(id.as_id(), Some(session_address), None)
            .await
            .into_data_error(id)?;

        if let Some(last_seen_time_update) = last_seen_time_update {
            self.update_last_seen_time(id.uuid, last_seen_time_update)
                .await;
        }

        Ok(())
    }

    pub async fn remove_tmp_files(&self, id: AccountIdInternal) -> Result<(), DataError> {
        self.files()
            .tmp_dir(id.into())
            .overwrite_and_remove_contents_if_exists()
            .await
            .into_data_error(id)
    }

    pub async fn set_is_bot_account(
        &self,
        id: AccountIdInternal,
        value: bool,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common().state().set_is_bot_account(id, value)
        })
    }

    pub async fn internal_handle_new_account_data_after_db_modification(
        &self,
        id: AccountIdInternal,
        current_account: &Account,
        new_account: &Account,
    ) -> Result<(), DataError> {
        let new_account_clone = new_account.clone();
        self.write_cache_common(id, |cache| {
            cache.permissions = new_account_clone.permissions();
            cache.account_state_related_shared_state = new_account_clone.into();
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
}

pub trait UpdateLocationIndexVisibility {
    async fn profile_update_location_index_visibility(
        &self,
        id: AccountIdInternal,
        visibility: bool,
    ) -> Result<(), DataError>;

    async fn update_last_seen_time(&self, account_id: AccountId, info: LastSeenTimeUpdated);
}

impl<I: InternalWriting> UpdateLocationIndexVisibility for I {
    async fn profile_update_location_index_visibility(
        &self,
        id: AccountIdInternal,
        visibility: bool,
    ) -> Result<(), DataError> {
        let (location, profile_data) = self
            .cache()
            .read_cache(id.as_id(), |e| {
                let index_data = e.location_index_profile_data()?;
                let p = e
                    .profile
                    .as_ref()
                    .ok_or(CacheError::FeatureNotEnabled.report())?;

                Ok::<
                    (
                        model_server_data::LocationIndexKey,
                        model_server_data::LocationIndexProfileData,
                    ),
                    error_stack::Report<CacheError>,
                >((p.location.current_position.profile_location(), index_data))
            })
            .await
            .into_data_error(id)?;

        if visibility {
            self.location_index_write_handle()
                .update_profile_data(id.as_id(), profile_data, location)
                .await?;
        } else {
            self.location_index_write_handle()
                .remove_profile_data(id.as_id(), location)
                .await?;
        }

        Ok(())
    }

    async fn update_last_seen_time(&self, account_id: AccountId, info: LastSeenTimeUpdated) {
        self.location_index_write_handle()
            .update_last_seen_time(account_id, info)
            .await
    }
}
