use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::AccountId;
use model_server_state::DemoAccountId;

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountDemo);

impl CurrentReadAccountDemo<'_> {
    pub fn owned_account_ids(
        &mut self,
        demo_account_id_value: DemoAccountId,
    ) -> Result<Vec<AccountId>, DieselDatabaseError> {
        use crate::schema::{account_id, demo_account_owned_accounts::dsl::*};

        demo_account_owned_accounts
            .inner_join(account_id::table)
            .filter(demo_account_id.eq(demo_account_id_value))
            .select(account_id::uuid)
            .load(self.conn())
            .into_db_error(demo_account_id_value)
    }
}
