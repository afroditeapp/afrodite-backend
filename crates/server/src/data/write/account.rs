use model::{
    Account, AccountData, AccountId, AccountIdInternal, AccountInternal, AccountSetup, AccountState, Capabilities, DemoModeId, ProfileLink, ProfileVisibility, SharedStateRaw
};
use simple_backend_database::diesel_db::DieselDatabaseError;

use super::db_transaction;
use crate::{data::{cache::CacheError, DataError, IntoDataError}, result::Result};

define_write_commands!(WriteCommandsAccount);

#[derive(Debug, Clone, Copy)]
pub struct IncrementAdminAccessGrantedCount;

impl WriteCommandsAccount<'_> {
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

        if self.config().components().profile &&
            current_account.profile_visibility().is_currently_public() != new_account.profile_visibility().is_currently_public() {
            self.profile_update_location_index_visibility(id, new_account.profile_visibility().is_currently_public()).await?;
        }

        Ok(())
    }

    /// The only method which can modify AccountState, Capabilities and
    /// ProfileVisibility. This also updates profile index if profile component
    /// is enabled and the visibility changed.
    ///
    /// Returns the modified Account.
    pub async fn update_syncable_account_data(
        &self,
        id: AccountIdInternal,
        increment_admin_access_granted: Option<IncrementAdminAccessGrantedCount>,
        modify_action: impl FnOnce(&mut AccountState, &mut Capabilities, &mut ProfileVisibility) -> error_stack::Result<(), DieselDatabaseError> + Send + 'static,
    ) -> Result<Account, DataError> {
        let current_account = self.db_read(move |mut cmds| cmds.common().account(id)).await?;
        let a = current_account.clone();
        let new_account = db_transaction!(self, move |mut cmds| {
            let account = cmds.common().state().update_syncable_account_data(id, a, modify_action)?;

            if increment_admin_access_granted.is_some() {
                cmds.account().data().upsert_increment_admin_access_granted_count()?;
            }

            Ok(account)
        })?;

        self.internal_handle_new_account_data_after_db_modification(id, &current_account, &new_account).await?;

        Ok(new_account)
    }

    /// Only server WebSocket code should call this method.
    pub async fn reset_syncable_account_data_version(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common().state().reset_account_data_version_number(id)
        })
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

                Ok((
                    p.location.current_position,
                    p.location_index_profile_data(),
                ))
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

    pub async fn account_setup(
        &self,
        id: AccountIdInternal,
        account_setup: AccountSetup,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().data().account_setup(id, &account_setup)
        })
    }

    pub async fn account_data(
        &self,
        id: AccountIdInternal,
        account_data: AccountData,
    ) -> Result<(), DataError> {
        let internal = AccountInternal {
            email: account_data.email,
        };

        db_transaction!(self, move |mut cmds| {
            cmds.account().data().account(id, &internal)
        })
    }

    pub async fn insert_demo_mode_related_account_ids(
        &self,
        id: DemoModeId,
        account_id: AccountId,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().demo_mode().insert_related_account_id(id, account_id)
        })
    }

    pub async fn set_is_bot_account(
        &self,
        id: AccountIdInternal,
        value: bool,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().sign_in_with().set_is_bot_account(id, value)
        })
    }
}
