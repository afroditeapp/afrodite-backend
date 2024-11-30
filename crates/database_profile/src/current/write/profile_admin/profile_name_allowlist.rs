use database::{
    current::read::GetDbReadCommandsCommon, define_current_write_commands, DieselDatabaseError,
};
use diesel::{insert_into, prelude::*, update, ExpressionMethods};
use error_stack::Result;
use model_profile::{AccountIdInternal, ProfileNameModerationState};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileAdminProfileNameAllowlist);

impl CurrentWriteProfileAdminProfileNameAllowlist<'_> {
    pub fn moderate_profile_name(
        &mut self,
        moderator_id: AccountIdInternal,
        name_owner_id: AccountIdInternal,
        name: String,
        accepted: bool,
    ) -> Result<ProfileNameModerationState, DieselDatabaseError> {
        use model::schema::{profile_name_allowlist, profile_state};

        let moderator_is_bot = self
            .read()
            .common()
            .state()
            .other_shared_state(moderator_id)?
            .is_bot_account;

        let next_state = if accepted {
            if moderator_is_bot {
                ProfileNameModerationState::AcceptedByBot
            } else {
                ProfileNameModerationState::AcceptedByHuman
            }
        } else if moderator_is_bot {
            ProfileNameModerationState::RejectedByBot
        } else {
            ProfileNameModerationState::RejectedByHuman
        };

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
        }

        update(profile_state::table)
            .filter(profile_state::account_id.eq(name_owner_id.as_db_id()))
            .set((profile_state::profile_name_moderation_state.eq(next_state),))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(next_state)
    }
}
