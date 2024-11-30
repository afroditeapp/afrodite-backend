use diesel::{insert_into, ExpressionMethods, RunQueryDsl};
use error_stack::Result;
use model::{AccountId, AccountIdDb, AccountIdInternal};
use simple_backend_database::diesel_db::DieselDatabaseError;

use crate::{define_current_write_commands, IntoDatabaseError};

mod queue_number;
mod state;
mod token;

define_current_write_commands!(CurrentWriteCommon);

impl<'a> CurrentWriteCommon<'a> {
    pub fn queue_number(self) -> queue_number::CurrentWriteCommonQueueNumber<'a> {
        queue_number::CurrentWriteCommonQueueNumber::new(self.cmds)
    }

    pub fn state(self) -> state::CurrentWriteCommonState<'a> {
        state::CurrentWriteCommonState::new(self.cmds)
    }

    pub fn token(self) -> token::CurrentWriteAccountToken<'a> {
        token::CurrentWriteAccountToken::new(self.cmds)
    }
}

impl CurrentWriteCommon<'_> {
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
