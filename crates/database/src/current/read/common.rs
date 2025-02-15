use diesel::{prelude::*, SelectableHelper};
use error_stack::Result;
use model::{Account, AccountId, AccountIdDb, AccountIdInternal, Permissions};
use simple_backend_database::diesel_db::DieselDatabaseError;

use crate::{
    current::read::GetDbReadCommandsCommon, define_current_read_commands, IntoDatabaseError,
};

mod queue_number;
mod state;
mod token;
mod report;

define_current_read_commands!(CurrentReadCommon);

impl<'a> CurrentReadCommon<'a> {
    pub fn state(self) -> state::CurrentReadCommonState<'a> {
        state::CurrentReadCommonState::new(self.cmds)
    }

    pub fn queue_number(self) -> queue_number::CurrentReadCommonQueueNumber<'a> {
        queue_number::CurrentReadCommonQueueNumber::new(self.cmds)
    }

    pub fn token(self) -> token::CurrentReadAccountToken<'a> {
        token::CurrentReadAccountToken::new(self.cmds)
    }

    pub fn report(self) -> report::CurrentReadCommonReport<'a> {
        report::CurrentReadCommonReport::new(self.cmds)
    }
}

impl CurrentReadCommon<'_> {
    /// This data is available on all servers as if microservice mode is
    /// enabled, the account server will update the data to other servers.
    pub fn account(&mut self, id: AccountIdInternal) -> Result<Account, DieselDatabaseError> {
        use crate::schema::account_permissions;

        let shared_state = self
            .read()
            .common()
            .state()
            .account_state_related_shared_state(id)?;

        let permissions: Permissions = account_permissions::table
            .filter(account_permissions::account_id.eq(id.as_db_id()))
            .select(Permissions::as_select())
            .first(self.conn())
            .into_db_error(id)?;

        Ok(Account::new_from_internal_types(permissions, shared_state))
    }

    pub fn account_ids_internal(&mut self) -> Result<Vec<AccountIdInternal>, DieselDatabaseError> {
        use crate::schema::account_id::dsl::*;

        account_id
            .select(AccountIdInternal::as_select())
            .load(self.conn())
            .into_db_error(())
    }

    pub fn account_ids(&mut self) -> Result<Vec<AccountId>, DieselDatabaseError> {
        use crate::schema::account_id::dsl::*;

        account_id.select(uuid).load(self.conn()).into_db_error(())
    }

    pub fn db_id_to_internal_id(&mut self, db_id: AccountIdDb) -> Result<AccountIdInternal, DieselDatabaseError> {
        use crate::schema::account_id::dsl::*;

        let uuid_value = account_id
            .filter(id.eq(db_id))
            .select(uuid)
            .first(self.conn())
            .into_db_error(())?;

        Ok(AccountIdInternal::new(db_id, uuid_value))
    }
}
