use config::csv::profile_name_allowlist::ProfileNameAllowlistData;
use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{prelude::*, update, ExpressionMethods};
use error_stack::Result;
use model_profile::{AccountIdInternal, ProfileNameModerationState};

use crate::{current::read::GetDbReadCommandsProfile, IntoDatabaseError};

define_current_write_commands!(CurrentWriteProfileNameAllowlist);

impl CurrentWriteProfileNameAllowlist<'_> {
    pub fn reset_profile_name_moderation_state(
        &mut self,
        id: AccountIdInternal,
        new_name: &str,
        ram_allowlist: &ProfileNameAllowlistData,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_state;

        let new_name = new_name.trim().to_lowercase();
        let name_accepted = ram_allowlist.name_exists(&new_name)
            || self
                .read()
                .profile()
                .profile_name_allowlist()
                .is_on_database_allowlist(&new_name)?;

        let new_state = if name_accepted {
            ProfileNameModerationState::AcceptedUsingAllowlist
        } else {
            ProfileNameModerationState::WaitingBotOrHumanModeration
        };

        update(profile_state::table)
            .filter(profile_state::account_id.eq(id.as_db_id()))
            .set((profile_state::profile_name_moderation_state.eq(new_state),))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
