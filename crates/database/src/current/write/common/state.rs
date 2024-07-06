use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{
    Account, AccountIdInternal, AccountState, AccountStateRelatedSharedState, AccountSyncVersion, Capabilities, OtherSharedState, ProfileVisibility, SharedStateRaw, SyncVersionUtils
};
use simple_backend_database::diesel_db::DieselDatabaseError;
use simple_backend_utils::ContextExt;

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_write_commands!(CurrentWriteAccountState, CurrentSyncWriteCommonState);

impl<C: ConnectionProvider> CurrentSyncWriteCommonState<C> {
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

    pub fn update_other_shared_state(
        &mut self,
        id: AccountIdInternal,
        data: OtherSharedState,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::shared_state::dsl::*;

        update(shared_state.find(id.as_db_id()))
            .set(data)
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn insert_default_account_capabilities(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_capabilities::dsl::*;

        insert_into(account_capabilities)
            .values((account_id.eq(id.as_db_id()),))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    fn account_capabilities(
        &mut self,
        id: AccountIdInternal,
        data: Capabilities,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_capabilities::dsl::*;

        update(account_capabilities.find(id.as_db_id()))
            .set(data)
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    /// The only method which can modify AccountState, Capabilities and
    /// ProfileVisibility. Updates automatically the AccountSyncVersion number.
    ///
    /// Returns the modified Account.
    pub fn update_syncable_account_data(
        &mut self,
        id: AccountIdInternal,
        account: Account,
        modify_action: impl FnOnce(
                &mut AccountState,
                &mut Capabilities,
                &mut ProfileVisibility,
            ) -> error_stack::Result<(), DieselDatabaseError>
            + Send
            + 'static,
    ) -> Result<Account, DieselDatabaseError> {
        let mut state = account.state();
        let mut capabilities = account.capablities();
        let mut profile_visibility = account.profile_visibility();
        modify_action(&mut state, &mut capabilities, &mut profile_visibility)
            .map_err(|_| DieselDatabaseError::NotAllowed.report())?;
        let new_version = account.sync_version().increment_if_not_max_value();
        let new_account = Account::new_from(capabilities, state, profile_visibility, new_version);

        self.account_capabilities(id, new_account.capablities())?;
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
        let mut shared_state: AccountStateRelatedSharedState = self.read().common().account(id)?.into();
        shared_state.sync_version = AccountSyncVersion::default();
        self.update_account_related_shared_state(id, shared_state)?;
        Ok(())
    }
}
