use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{
    Account, AccountIdInternal, AccountStateContainer, AccountStateRelatedSharedState,
    AccountSyncVersion, BotAccountType, InitialSetupCompletedTime, Permissions, ProfileVisibility,
    SharedStateRaw, SyncVersionUtils,
};
use simple_backend_database::diesel_db::DieselDatabaseError;
use simple_backend_utils::ContextExt;

use crate::{
    IntoDatabaseError, current::read::GetDbReadCommandsCommon, define_current_write_commands,
};

define_current_write_commands!(CurrentWriteCommonState);

pub struct AccountUpdate {
    pub state: AccountStateContainer,
    pub permissions: Permissions,
    pub profile_visibility: ProfileVisibility,
    pub email_verified: bool,
    pub age_verified: bool,
}

#[must_use = "Account returned from DB update should be used to update cache before dropping"]
pub struct CacheUpdateAccount(Account);

impl CacheUpdateAccount {
    pub fn into_inner(self) -> Account {
        self.0
    }
}

impl AsRef<Account> for CacheUpdateAccount {
    fn as_ref(&self) -> &Account {
        &self.0
    }
}

impl CurrentWriteCommonState<'_> {
    pub fn insert_shared_state(
        &mut self,
        id: AccountIdInternal,
        data: SharedStateRaw,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::shared_state::dsl::*;

        insert_into(shared_state)
            .values((account_id.eq(id.as_db_id()), data))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    fn update_account_related_shared_state(
        &mut self,
        id: AccountIdInternal,
        data: AccountStateRelatedSharedState,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::shared_state::dsl::*;

        update(shared_state.find(id.as_db_id()))
            .set(data)
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn update_unlimited_likes(
        &mut self,
        id: AccountIdInternal,
        unlimited_likes_value: bool,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::shared_state::dsl::*;

        update(shared_state.find(id.as_db_id()))
            .set(unlimited_likes.eq(unlimited_likes_value))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn set_bot_account_type_number(
        &mut self,
        id: AccountIdInternal,
        value_for_bot_account_type_number: BotAccountType,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::shared_state::dsl::*;

        update(shared_state.find(id.as_db_id()))
            .set(bot_account_type_number.eq(Some(value_for_bot_account_type_number)))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn insert_default_account_permissions(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_permissions::dsl::*;

        insert_into(account_permissions)
            .values((account_id.eq(id.as_db_id()),))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    fn account_permissions(
        &mut self,
        id: AccountIdInternal,
        data: Permissions,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_permissions::dsl::*;

        update(account_permissions.find(id.as_db_id()))
            .set(data)
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    /// The only method which can modify [Account].
    /// Updates automatically the AccountSyncVersion number.
    ///
    /// Returns the modified [Account] wrapped in [CacheUpdateAccount].
    pub fn update_syncable_account_data(
        &mut self,
        id: AccountIdInternal,
        account: Account,
        modify_action: impl FnOnce(&mut AccountUpdate) -> error_stack::Result<(), DieselDatabaseError>
        + Send
        + 'static,
    ) -> Result<CacheUpdateAccount, DieselDatabaseError> {
        let mut account_mut = AccountUpdate {
            state: account.state_container(),
            permissions: account.permissions(),
            profile_visibility: account.profile_visibility_raw(),
            email_verified: account.email_verified(),
            age_verified: account.age_verified(),
        };
        modify_action(&mut account_mut).map_err(|_| DieselDatabaseError::NotAllowed.report())?;
        let new_version = account.sync_version().increment_if_not_max_value();
        let new_account = Account::new_from(
            account_mut.permissions,
            account_mut.state,
            account_mut.profile_visibility,
            account_mut.email_verified,
            account_mut.age_verified,
            new_version,
        );

        self.account_permissions(id, new_account.permissions())?;
        self.update_account_related_shared_state(id, new_account.clone().into())?;

        Ok(CacheUpdateAccount(new_account))
    }

    /// Reset Account data version number to 0.
    ///
    /// Only the WebSocket code must modify the version number.
    pub fn reset_account_data_version_number(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        let mut shared_state: AccountStateRelatedSharedState =
            self.read().common().account(id)?.into();
        shared_state.sync_version = AccountSyncVersion::default();
        self.update_account_related_shared_state(id, shared_state)?;
        Ok(())
    }

    pub fn update_initial_setup_completed_unix_time(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<InitialSetupCompletedTime, DieselDatabaseError> {
        use model::schema::shared_state::dsl::*;

        let current_time = InitialSetupCompletedTime::current_time();

        update(shared_state.find(id.as_db_id()))
            .set(initial_setup_completed_unix_time.eq(current_time))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(current_time)
    }
}
