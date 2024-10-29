use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*, update, ExpressionMethods};
use error_stack::Result;
use model::AccountIdInternal;

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileAdminProfileNameAllowlist, CurrentWriteProfileAdminProfileNameAllowlist);

impl<C: ConnectionProvider> CurrentWriteProfileAdminProfileNameAllowlist<C> {
    pub fn moderate_profile_name(
        &mut self,
        moderator_id: AccountIdInternal,
        name_owner_id: AccountIdInternal,
        name: String,
        accepted: bool,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::{profile_name_allowlist, profile, profile_state};

        if accepted {
            let allowlist_name = name.trim().to_lowercase();
            insert_into(profile_name_allowlist::table)
                .values((
                    profile_name_allowlist::profile_name.eq(allowlist_name),
                    profile_name_allowlist::name_creator_account_id.eq(name_owner_id.as_db_id()),
                    profile_name_allowlist::name_moderator_account_id.eq(moderator_id.as_db_id()),
                ))
                .on_conflict(profile_name_allowlist::profile_name)
                .do_nothing()
                .execute(self.conn())
                .into_db_error(())?;
            update(profile::table)
                .filter(profile::account_id.eq(name_owner_id.as_db_id()))
                .set((
                    profile::name_accepted.eq(true),
                ))
                .execute(self.conn())
                .into_db_error(())?;
        } else {
            update(profile_state::table)
                .filter(profile_state::account_id.eq(name_owner_id.as_db_id()))
                .set((
                    profile_state::profile_name_denied.eq(true),
                ))
                .execute(self.conn())
                .into_db_error(())?;
        }

        Ok(())
    }
}
