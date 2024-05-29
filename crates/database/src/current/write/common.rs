use diesel::{insert_into, ExpressionMethods, RunQueryDsl};
use model::{AccountId, AccountIdDb, AccountIdInternal};
use simple_backend_database::diesel_db::DieselDatabaseError;
use error_stack::Result;
use crate::IntoDatabaseError;

use super::ConnectionProvider;

mod queue_number;
mod state;
mod token;

define_write_commands!(CurrentWriteAccount, CurrentSyncWriteCommon);

impl<C: ConnectionProvider> CurrentSyncWriteCommon<C> {
    pub fn queue_number(self) -> queue_number::CurrentSyncWriteCommonQueueNumber<C> {
        queue_number::CurrentSyncWriteCommonQueueNumber::new(self.cmds)
    }

    pub fn state(self) -> state::CurrentSyncWriteCommonState<C> {
        state::CurrentSyncWriteCommonState::new(self.cmds)
    }

    pub fn token(self) -> token::CurrentSyncWriteAccountToken<C> {
        token::CurrentSyncWriteAccountToken::new(self.cmds)
    }

    pub fn insert_account_id(
        mut self,
        account_uuid: AccountId,
    ) -> Result<AccountIdInternal, DieselDatabaseError> {
        use model::schema::account_id::dsl::*;

        let db_id: AccountIdDb = insert_into(account_id)
            .values(uuid.eq(account_uuid))
            .returning(id)
            .get_result(self.conn())
            .into_db_error(account_uuid)?;

        Ok(AccountIdInternal {
            uuid: account_uuid,
            id: db_id,
        })
    }
}
