use diesel::{prelude::*, SelectableHelper};
use error_stack::Result;
use model::{Account, AccountIdInternal, Capabilities};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

mod queue_number;
mod state;
mod token;

define_read_commands!(CurrentReadAccount, CurrentSyncReadCommon);

impl<C: ConnectionProvider> CurrentSyncReadCommon<C> {
    pub fn state(self) -> state::CurrentSyncReadCommonState<C> {
        state::CurrentSyncReadCommonState::new(self.cmds)
    }

    pub fn queue_number(self) -> queue_number::CurrentSyncReadCommonQueueNumber<C> {
        queue_number::CurrentSyncReadCommonQueueNumber::new(self.cmds)
    }

    pub fn token(self) -> token::CurrentSyncReadAccountToken<C> {
        token::CurrentSyncReadAccountToken::new(self.cmds)
    }

    /// This data is available on all servers as if microservice mode is
    /// enabled, the account server will update the data to other servers.
    pub fn account(&mut self, id: AccountIdInternal) -> Result<Account, DieselDatabaseError> {
        use crate::schema::account_capabilities;

        let shared_state = self.read().common().state().account_state_related_shared_state(id)?;

        let capabilities: Capabilities = account_capabilities::table
            .filter(account_capabilities::account_id.eq(id.as_db_id()))
            .select(Capabilities::as_select())
            .first(self.conn())
            .into_db_error(id)?;

        Ok(Account::new_from_internal_types(capabilities, shared_state))
    }
}
