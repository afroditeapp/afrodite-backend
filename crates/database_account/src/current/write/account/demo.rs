use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::AccountIdInternal;
use model_server_state::DemoAccountId;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountDemo);

impl CurrentWriteAccountDemo<'_> {
    pub fn add_to_demo_account_owned_accounts(
        &mut self,
        demo_account_id_value: DemoAccountId,
        account: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::demo_account_owned_accounts::dsl::*;

        insert_into(demo_account_owned_accounts)
            .values((
                demo_account_id.eq(demo_account_id_value),
                account_id.eq(account.as_db_id()),
            ))
            .execute(self.conn())
            .into_db_error(account)?;

        Ok(())
    }
}
