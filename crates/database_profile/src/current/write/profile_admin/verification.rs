use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{prelude::*, update};
use error_stack::Result;
use model::AccountIdInternal;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileAdminVerification);

impl CurrentWriteProfileAdminVerification<'_> {
    pub fn change_profile_age_range_verified_value(
        &mut self,
        account: AccountIdInternal,
        value: Option<bool>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_state;

        update(profile_state::table)
            .filter(profile_state::account_id.eq(account.as_db_id()))
            .set(profile_state::profile_age_range_verified.eq(value))
            .execute(self.conn())
            .into_db_error(account)?;

        Ok(())
    }

    pub fn change_profile_age_range_verified_manual_value(
        &mut self,
        account: AccountIdInternal,
        value: Option<bool>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_state;

        update(profile_state::table)
            .filter(profile_state::account_id.eq(account.as_db_id()))
            .set(profile_state::profile_age_range_verified_manual.eq(value))
            .execute(self.conn())
            .into_db_error(account)?;

        Ok(())
    }

    pub fn change_profile_name_verified_value(
        &mut self,
        account: AccountIdInternal,
        value: Option<bool>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_state;

        update(profile_state::table)
            .filter(profile_state::account_id.eq(account.as_db_id()))
            .set(profile_state::profile_name_verified.eq(value))
            .execute(self.conn())
            .into_db_error(account)?;

        Ok(())
    }

    pub fn change_profile_name_verified_manual_value(
        &mut self,
        account: AccountIdInternal,
        value: Option<bool>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_state;

        update(profile_state::table)
            .filter(profile_state::account_id.eq(account.as_db_id()))
            .set(profile_state::profile_name_verified_manual.eq(value))
            .execute(self.conn())
            .into_db_error(account)?;

        Ok(())
    }
}
