use config::csv::profile_name_allowlist::ProfileNameAllowlistData;
use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{ExpressionMethods, delete, insert_into, prelude::*, upsert::excluded};
use error_stack::Result;
use model_profile::{
    AccountIdDb, AccountIdInternal, ProfileNameModerationState, ProfileStringModerationContentType,
    ProfileStringModerationState, ProfileTextModerationState, UnixTime,
};

use crate::{IntoDatabaseError, current::read::GetDbReadCommandsProfile};

define_current_write_commands!(CurrentWriteModeration);

impl CurrentWriteModeration<'_> {
    pub fn reset_profile_name_moderation_state(
        &mut self,
        id: AccountIdInternal,
        new_name: &str,
        ram_allowlist: &ProfileNameAllowlistData,
    ) -> Result<ProfileNameModerationState, DieselDatabaseError> {
        use model::schema::profile_moderation::dsl::*;

        let new_name = new_name.trim().to_lowercase();
        let name_accepted = ram_allowlist.name_exists(&new_name)
            || self
                .read()
                .profile()
                .moderation()
                .is_name_on_database_allowlist(&new_name)?;

        let new_state = if name_accepted {
            ProfileStringModerationState::AcceptedByAllowlist
        } else {
            ProfileStringModerationState::WaitingBotOrHumanModeration
        };

        insert_into(profile_moderation)
            .values((
                account_id.eq(id.as_db_id()),
                content_type.eq(ProfileStringModerationContentType::ProfileName),
                state_type.eq(new_state),
                created_unix_time.eq(UnixTime::current_time()),
            ))
            .on_conflict((account_id, content_type))
            .do_update()
            .set((
                state_type.eq(excluded(state_type)),
                rejected_reason_category.eq(None::<i64>),
                rejected_reason_details.eq(String::new()),
                moderator_account_id.eq(None::<AccountIdDb>),
                created_unix_time.eq(excluded(created_unix_time)),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(ProfileNameModerationState(new_state))
    }

    pub fn reset_profile_text_moderation_state(
        &mut self,
        id: AccountIdInternal,
        text_empty: bool,
    ) -> Result<Option<ProfileTextModerationState>, DieselDatabaseError> {
        use model::schema::profile_moderation::dsl::*;

        if text_empty {
            delete(profile_moderation)
                .filter(account_id.eq(id.as_db_id()))
                .filter(content_type.eq(ProfileStringModerationContentType::ProfileText))
                .execute(self.conn())
                .into_db_error(id)?;
            Ok(None)
        } else {
            let new_state = ProfileStringModerationState::WaitingBotOrHumanModeration;

            insert_into(profile_moderation)
                .values((
                    account_id.eq(id.as_db_id()),
                    content_type.eq(ProfileStringModerationContentType::ProfileText),
                    state_type.eq(new_state),
                    created_unix_time.eq(UnixTime::current_time()),
                ))
                .on_conflict((account_id, content_type))
                .do_update()
                .set((
                    state_type.eq(excluded(state_type)),
                    rejected_reason_category.eq(None::<i64>),
                    rejected_reason_details.eq(String::new()),
                    moderator_account_id.eq(None::<AccountIdDb>),
                    created_unix_time.eq(excluded(created_unix_time)),
                ))
                .execute(self.conn())
                .into_db_error(id)?;

            Ok(Some(ProfileTextModerationState(new_state)))
        }
    }
}
