use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use model::AccountIdInternal;
use model_account::{
    AccountGlobalState, AccountStateTableRaw, EmailAddress, EmailAddressState,
    EmailAddressStateInternal, EmailChange,
};
use simple_backend_utils::Result;

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountData);

impl CurrentReadAccountData<'_> {
    pub fn email_address(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<EmailAddress>, DieselDatabaseError> {
        use crate::schema::account_email_address_state::dsl::*;

        let email_value: Option<EmailAddress> = account_email_address_state
            .filter(account_id.eq(id.as_db_id()))
            .select(email)
            .first(self.conn())
            .into_db_error(id)?;

        Ok(email_value)
    }

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

        let email_change = self.email_change(id)?;

        Ok(EmailAddressState::new(internal, email_change))
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

    pub fn email_change(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<EmailChange>, DieselDatabaseError> {
        use crate::schema::account_email_change::dsl::*;

        let data: Option<EmailChange> = account_email_change
            .filter(account_id.eq(id.as_db_id()))
            .select(EmailChange::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(id)?;

        Ok(data)
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
