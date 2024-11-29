use database::{define_current_read_commands, DieselDatabaseError, IntoDatabaseError};
use diesel::{query_dsl::methods::{FilterDsl, SelectDsl}, ExpressionMethods, RunQueryDsl};
use model::AccountIdInternal;
use model_account::ProfileNameAndAge;

use error_stack::Result;

define_current_read_commands!(CurrentReadProfileUtils);

impl CurrentReadProfileUtils<'_> {
    pub fn profile_name_and_age(&mut self, id: AccountIdInternal) -> Result<ProfileNameAndAge, DieselDatabaseError> {
        use crate::schema::profile::dsl::*;

        let (name_value, age_value) = profile
            .filter(account_id.eq(id.as_db_id()))
            .select((name, age))
            .first(self.conn())
            .into_db_error(())?;

        Ok(ProfileNameAndAge {
            name: name_value,
            age: age_value,
        })
    }
}
