use database::{define_current_read_commands, DieselDatabaseError, IntoDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model_profile::{
    GetProfileNamePendingModerationList, ProfileNameModerationState, ProfileNamePendingModeration,
};

define_current_read_commands!(CurrentReadProfileNameAllowlist);

impl CurrentReadProfileNameAllowlist<'_> {
    pub fn profile_name_pending_moderation_list(
        &mut self,
    ) -> Result<GetProfileNamePendingModerationList, DieselDatabaseError> {
        use crate::schema::{account_id, profile::dsl::*, profile_state};

        let values = profile
            .inner_join(account_id::table)
            .inner_join(profile_state::table.on(profile_state::account_id.eq(account_id::id)))
            .filter(
                profile_state::profile_name_moderation_state
                    .eq(ProfileNameModerationState::WaitingBotOrHumanModeration)
                    .or(profile_state::profile_name_moderation_state
                        .eq(ProfileNameModerationState::WaitingHumanModeration)),
            )
            .select((account_id::uuid, name))
            .order(account_id::id.asc())
            .load::<ProfileNamePendingModeration>(self.conn())
            .into_db_error(())?;

        Ok(GetProfileNamePendingModerationList { values })
    }
}
