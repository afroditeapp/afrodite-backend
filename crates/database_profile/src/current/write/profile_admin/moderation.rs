use database::{
    DieselDatabaseError, current::read::GetDbReadCommandsCommon, define_current_write_commands,
};
use diesel::{ExpressionMethods, delete, insert_into, prelude::*, update};
use error_stack::Result;
use model_profile::{
    AccountIdInternal, ProfileStringModerationContentType,
    ProfileStringModerationRejectedReasonCategory, ProfileStringModerationRejectedReasonDetails,
    ProfileStringModerationState,
};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileAdminModeration);

impl CurrentWriteProfileAdminModeration<'_> {
    pub fn add_to_profile_name_allowlist(
        &mut self,
        moderator_id: AccountIdInternal,
        name_owner_id: AccountIdInternal,
        name: String,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_name_allowlist::dsl::*;
        let allowlist_name = name.trim().to_lowercase();
        insert_into(profile_name_allowlist)
            .values((
                profile_name.eq(allowlist_name),
                name_creator_account_id.eq(name_owner_id.as_db_id()),
                name_moderator_account_id.eq(moderator_id.as_db_id()),
            ))
            .on_conflict(profile_name)
            .do_nothing()
            .execute(self.conn())
            .into_db_error(())?;
        Ok(())
    }

    pub fn delete_from_profile_name_allowlist(
        &mut self,
        name: String,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_name_allowlist::dsl::*;
        let allowlist_name = name.trim().to_lowercase();
        delete(profile_name_allowlist)
            .filter(profile_name.eq(allowlist_name))
            .execute(self.conn())
            .into_db_error(())?;
        Ok(())
    }

    pub fn moderate_profile_string(
        &mut self,
        moderator_id: AccountIdInternal,
        string_owner_id: AccountIdInternal,
        content_type_value: ProfileStringModerationContentType,
        accepted: bool,
        rejected_category: Option<ProfileStringModerationRejectedReasonCategory>,
        rejected_details: Option<ProfileStringModerationRejectedReasonDetails>,
    ) -> Result<ProfileStringModerationState, DieselDatabaseError> {
        let moderator_is_bot = self
            .read()
            .common()
            .state()
            .other_shared_state(moderator_id)?
            .is_bot_account;

        let next_state = if accepted {
            if moderator_is_bot {
                ProfileStringModerationState::AcceptedByBot
            } else {
                ProfileStringModerationState::AcceptedByHuman
            }
        } else if moderator_is_bot {
            ProfileStringModerationState::RejectedByBot
        } else {
            ProfileStringModerationState::RejectedByHuman
        };

        {
            use model::schema::profile_moderation::dsl::*;
            update(profile_moderation)
                .filter(account_id.eq(string_owner_id.as_db_id()))
                .filter(content_type.eq(content_type_value))
                .set((
                    state_type.eq(next_state),
                    rejected_reason_category.eq(rejected_category),
                    rejected_reason_details.eq(rejected_details),
                    moderator_account_id.eq(moderator_id.as_db_id()),
                ))
                .execute(self.conn())
                .into_db_error(())?;
        }

        Ok(next_state)
    }

    pub fn move_to_human_moderation(
        &mut self,
        id: AccountIdInternal,
        content_type_value: ProfileStringModerationContentType,
    ) -> Result<ProfileStringModerationState, DieselDatabaseError> {
        use model::schema::profile_moderation::dsl::*;

        let next_state = ProfileStringModerationState::WaitingHumanModeration;

        update(profile_moderation)
            .filter(account_id.eq(id.as_db_id()))
            .filter(content_type.eq(content_type_value))
            .set((state_type.eq(next_state),))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(next_state)
    }
}
