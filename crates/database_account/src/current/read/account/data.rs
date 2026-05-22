use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::AccountIdInternal;
use model_account::{
    AccountGlobalState, AccountStateTableRaw, EmailAddressState, EmailAddressStateInternal,
};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountData);

impl CurrentReadAccountData<'_> {
    pub fn email_address_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<EmailAddressState, DieselDatabaseError> {
        use crate::schema::account_email_address_state::dsl::*;

        let internal = account_email_address_state
            .filter(account_id.eq(id.as_db_id()))
            .select(EmailAddressStateInternal::as_select())
            .first(self.conn())
            .into_db_error(id)?;

        Ok(EmailAddressState {
            email: internal.email,
            email_change: internal.email_change,
            email_change_verified: internal.email_change_verified,
            email_change_completion_time: None,
            email_login_enabled: internal.email_login_enabled,
        })
    }

    pub fn email_address_state_internal(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<EmailAddressStateInternal, DieselDatabaseError> {
        use crate::schema::account_email_address_state::dsl::*;
        account_email_address_state
            .filter(account_id.eq(id.as_db_id()))
            .select(EmailAddressStateInternal::as_select())
            .first(self.conn())
            .into_db_error(id)
    }

    pub fn account_state_table_raw(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AccountStateTableRaw, DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        account_state
            .filter(account_id.eq(id.as_db_id()))
            .select(AccountStateTableRaw::as_select())
            .first(self.conn())
            .into_db_error(id)
    }

    pub fn global_state(&mut self) -> Result<AccountGlobalState, DieselDatabaseError> {
        use model::schema::account_global_state::dsl::*;

        account_global_state
            .filter(row_type.eq(AccountGlobalState::ACCOUNT_GLOBAL_STATE_ROW_TYPE))
            .select(AccountGlobalState::as_select())
            .first(self.conn())
            .optional()
            .map(|v| v.unwrap_or_default())
            .into_db_error(())
    }
}
