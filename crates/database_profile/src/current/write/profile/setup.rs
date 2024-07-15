use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*, update, ExpressionMethods};
use error_stack::Result;
use model::{AccountIdInternal, SetProfileSetup};

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileFavorite, CurrentSyncWriteProfileSetup);

impl<C: ConnectionProvider> CurrentSyncWriteProfileSetup<C> {
    pub fn insert_default_profile_setup(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_setup::dsl::*;

        insert_into(profile_setup)
            .values(account_id.eq(id.as_db_id()))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn profile_setup(
        &mut self,
        id: AccountIdInternal,
        data: &SetProfileSetup,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_setup::dsl::*;

        update(profile_setup.find(id.as_db_id()))
            .set(birthdate.eq(data.birthdate))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
