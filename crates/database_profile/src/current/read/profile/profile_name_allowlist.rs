use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};

define_current_read_commands!(CurrentReadProfileNameAllowlist);

impl CurrentReadProfileNameAllowlist<'_> {
    pub fn is_on_database_allowlist(
        &mut self,
        name: &str,
    ) -> Result<bool, DieselDatabaseError> {
        use crate::schema::profile_name_allowlist::dsl::*;

        let exists = profile_name_allowlist
            .filter(profile_name.eq(&name))
            .select(name_creator_account_id)
            .first::<i64>(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)?;

        Ok(exists.is_some())
    }
}
