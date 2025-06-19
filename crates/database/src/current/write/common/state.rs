use chrono::NaiveDate;
use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{
    Account, AccountIdInternal, AccountStateContainer, AccountStateRelatedSharedState,
    AccountSyncVersion, InitialSetupCompletedTime, Permissions, ProfileVisibility, SharedStateRaw,
    SyncVersionUtils,
};
use simple_backend_database::diesel_db::DieselDatabaseError;
use simple_backend_utils::ContextExt;

use crate::{
    IntoDatabaseError, current::read::GetDbReadCommandsCommon, define_current_write_commands,
};

define_current_write_commands!(CurrentWriteCommonState);

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

    pub fn update_birthdate(
        &mut self,
        id: AccountIdInternal,
        birthdate_value: NaiveDate,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::shared_state::dsl::*;

        update(shared_state.find(id.as_db_id()))
            .set(birthdate.eq(birthdate_value))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn set_is_bot_account(
        &mut self,
        id: AccountIdInternal,
        value_for_is_bot_account: bool,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::shared_state::dsl::*;

        update(shared_state.find(id.as_db_id()))
            .set((is_bot_account.eq(value_for_is_bot_account),))
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

    /// The only method which can modify AccountStateContainer, Permissions and
    /// ProfileVisibility. Updates automatically the AccountSyncVersion number.
    ///
    /// Returns the modified Account.
    pub fn update_syncable_account_data(
        &mut self,
        id: AccountIdInternal,
        account: Account,
        modify_action: impl FnOnce(
            &mut AccountStateContainer,
            &mut Permissions,
            &mut ProfileVisibility,
        ) -> error_stack::Result<(), DieselDatabaseError>
        + Send
        + 'static,
    ) -> Result<Account, DieselDatabaseError> {
        let mut state = account.state_container();
        let mut permissions = account.permissions();
        let mut profile_visibility = account.profile_visibility();
        modify_action(&mut state, &mut permissions, &mut profile_visibility)
            .map_err(|_| DieselDatabaseError::NotAllowed.report())?;
        let new_version = account.sync_version().increment_if_not_max_value();
        let new_account = Account::new_from(permissions, state, profile_visibility, new_version);

        self.account_permissions(id, new_account.permissions())?;
        self.update_account_related_shared_state(id, new_account.clone().into())?;

        Ok(new_account)
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
