use database::{DieselDatabaseError, IntoDatabaseError, define_current_read_commands};
use diesel::{
    ExpressionMethods, RunQueryDsl,
    query_dsl::methods::{FilterDsl, SelectDsl},
};
use error_stack::Result;
use model::AccountIdInternal;
use model_account::ProfileNameAndAge;

define_current_read_commands!(CurrentReadProfileUtils);

impl CurrentReadProfileUtils<'_> {
    pub fn profile_name_and_age(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<ProfileNameAndAge, DieselDatabaseError> {
        use crate::schema::profile::dsl::*;

        let (name_value, age_value) = profile
            .filter(account_id.eq(id.as_db_id()))
            .select((profile_name, age))
            .first(self.conn())
            .into_db_error(())?;

        Ok(ProfileNameAndAge {
            name: name_value,
            age: age_value,
        })
    }
}
