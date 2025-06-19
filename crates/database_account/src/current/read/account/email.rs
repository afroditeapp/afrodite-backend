use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::AccountIdInternal;
use model_account::AccountEmailSendingStateRaw;

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountEmail);

impl CurrentReadAccountEmail<'_> {
    pub fn email_sending_states(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AccountEmailSendingStateRaw, DieselDatabaseError> {
        use crate::schema::account_email_sending_state::dsl::*;

        account_email_sending_state
            .filter(account_id.eq(id.as_db_id()))
            .select(AccountEmailSendingStateRaw::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(id)
            .map(|data| data.unwrap_or_default())
    }
}
