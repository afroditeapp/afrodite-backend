use config::profile_name_allowlist::ProfileNameAllowlistData;
use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{prelude::*, update, ExpressionMethods};
use error_stack::Result;
use model::AccountIdInternal;

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileNameAllowlist, CurrentSyncWriteProfileNameAllowlist);

impl<C: ConnectionProvider> CurrentSyncWriteProfileNameAllowlist<C> {
    pub fn reset_profile_name_accepted_and_denied_values(
        &mut self,
        id: AccountIdInternal,
        new_name: &str,
        ram_allowlist: &ProfileNameAllowlistData,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::{profile_state, profile};

        let new_name = new_name.trim().to_lowercase();
        let name_accepted = ram_allowlist.name_exists(&new_name) ||
            self.read()
                .profile()
                .profile_name_allowlist()
                .is_on_database_allowlist(&new_name)?;

        update(profile_state::table)
            .filter(profile_state::account_id.eq(id.as_db_id()))
            .set((
                profile_state::profile_name_denied.eq(false),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        update(profile::table)
            .filter(profile::account_id.eq(id.as_db_id()))
            .set((
                profile::name_accepted.eq(name_accepted),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
