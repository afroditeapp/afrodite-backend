use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use model::{AccountId, AccountIdInternal};
use model_server_state::DemoAccountId;
use simple_backend_utils::Result;

use crate::{IntoDatabaseError, schema::demo_account_owned_accounts::dsl::*};

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

    pub fn is_account_owned_by_demo_account(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<bool, DieselDatabaseError> {
        demo_account_owned_accounts
            .filter(account_id.eq(id.as_db_id()))
            .count()
            .get_result::<i64>(self.conn())
            .map(|count| count > 0)
            .into_db_error(id)
    }
}
