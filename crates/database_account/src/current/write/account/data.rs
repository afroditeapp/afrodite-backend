use database::{
    DieselDatabaseError, current::write::GetDbWriteCommandsCommon, define_current_write_commands,
};
use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{AccountCreatedTime, AccountId, AccountIdInternal, ClientId};
use model_account::{AccountGlobalState, EmailAddress, EmailAddressStateInternal, SetAccountSetup};
use simple_backend_utils::db::MyRunQueryDsl;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountData);

impl CurrentWriteAccountData<'_> {
    pub fn insert_email_address_state(
        &mut self,
        id: AccountIdInternal,
        internal: EmailAddressStateInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_email_address_state::dsl::*;

        insert_into(account_email_address_state)
            .values((account_id.eq(id.as_db_id()), email.eq(internal.email)))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn insert_default_account_setup(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_setup::dsl::*;

        insert_into(account_setup)
            .values(account_id.eq(id.as_db_id()))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn account_setup(
        &mut self,
        id: AccountIdInternal,
        data: &SetAccountSetup,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_setup::dsl::*;

        if let Some(birthdate_value) = &data.birthdate {
            self.write()
                .common()
                .state()
                .update_birthdate(id, *birthdate_value)?;
        }

        update(account_setup.find(id.as_db_id()))
            .set((
                birthdate.eq(data.birthdate),
                is_adult.eq(Some(data.is_adult)),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn insert_account_state(
        mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        insert_into(account_state)
            .values((account_id.eq(id.as_db_id()),))
            .execute(self.conn())
            .into_db_error(())?;
        Ok(())
    }

    pub fn upsert_increment_admin_access_granted_count(
        &mut self,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_global_state::dsl::*;

        insert_into(account_global_state)
            .values((
                row_type.eq(AccountGlobalState::ACCOUNT_GLOBAL_STATE_ROW_TYPE),
                admin_access_granted_count.eq(1),
            ))
            .on_conflict(row_type)
            .do_update()
            .set(admin_access_granted_count.eq(admin_access_granted_count + 1))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    /// Does not set email verification status to false as that
    /// needs sync version change. That should be done before calling
    /// this method.
    ///
    /// Does not clear email_verification_token_unix_time to limit
    /// verification email sending.
    pub fn update_account_email(
        mut self,
        id: AccountIdInternal,
        email_address: &EmailAddress,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_email_address_state::dsl::*;

        update(account_email_address_state.find(id.as_db_id()))
            .set((
                email.eq(email_address),
                email_verification_token.eq(None::<Vec<u8>>),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn get_next_client_id(
        mut self,
        id: AccountIdInternal,
    ) -> Result<ClientId, DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        let current: ClientId = account_state
            .filter(account_id.eq(id.as_db_id()))
            .select(next_client_id)
            .first(self.conn())
            .optional()
            .into_db_error(())?
            .unwrap_or_default();

        let next = current.increment();

        insert_into(account_state)
            .values((account_id.eq(id.as_db_id()), next_client_id.eq(next)))
            .on_conflict(account_id)
            .do_update()
            .set(next_client_id.eq(next))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(current)
    }

    pub fn new_unique_account_id(&mut self) -> Result<AccountIdInternal, DieselDatabaseError> {
        use model::schema::used_account_ids::dsl::*;

        let random_aid = AccountId::new_random();

        let db_id = insert_into(used_account_ids)
            .values(uuid.eq(random_aid))
            .returning(id)
            .get_result(self.conn())
            .into_db_error(random_aid)?;

        Ok(AccountIdInternal {
            id: db_id,
            uuid: random_aid,
        })
    }

    pub fn update_account_created_unix_time(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        let current_time = AccountCreatedTime::current_time();

        update(account_state.find(id.as_db_id()))
            .set(account_created_unix_time.eq(current_time))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
