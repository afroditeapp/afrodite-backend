use database::{DieselDatabaseError, IntoDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model_profile::{
    ProfileStringModerationContentType, ProfileStringModerationQueuePage,
    ProfileStringModerationQueueType, ProfileStringModerationState, ProfileStringPendingModeration,
};

define_current_read_commands!(CurrentReadProfileModeration);

impl CurrentReadProfileModeration<'_> {
    pub fn profile_string_moderation_page(
        &mut self,
        content_type: ProfileStringModerationContentType,
        queue_type: ProfileStringModerationQueueType,
    ) -> Result<ProfileStringModerationQueuePage, DieselDatabaseError> {
        use crate::schema::{account_id, profile, profile_moderation};

        const LIMIT: i64 = 25;

        let states = match queue_type {
            ProfileStringModerationQueueType::WaitingAdminBot => {
                [ProfileStringModerationState::WaitingAdminBot].as_slice()
            }
            ProfileStringModerationQueueType::WaitingAdmin => {
                [ProfileStringModerationState::WaitingAdmin].as_slice()
            }
            ProfileStringModerationQueueType::AcceptedByAdminBot => {
                [ProfileStringModerationState::AcceptedByAdminBot].as_slice()
            }
            ProfileStringModerationQueueType::RejectedByAdminBot => {
                [ProfileStringModerationState::RejectedByAdminBot].as_slice()
            }
        };

        if states.is_empty() {
            return Ok(ProfileStringModerationQueuePage { values: vec![] });
        }

        let query = profile::table
            .inner_join(account_id::table)
            .inner_join(
                profile_moderation::table.on(profile_moderation::account_id.eq(account_id::id)),
            )
            .filter(profile_moderation::content_type.eq(content_type))
            .filter(profile_moderation::state_type.eq_any(states))
            .order((
                profile_moderation::created_unix_time.asc(),
                account_id::id.asc(),
            ))
            .limit(LIMIT);

        let values = match content_type {
            ProfileStringModerationContentType::ProfileName => query
                .filter(profile::profile_name.is_not_null())
                .select((
                    account_id::uuid,
                    profile::profile_name.assume_not_null(),
                    profile_moderation::rejected_reason_category,
                    profile_moderation::rejected_reason_details,
                ))
                .load::<ProfileStringPendingModeration>(self.conn()),
            ProfileStringModerationContentType::ProfileText => query
                .filter(profile::profile_text.is_not_null())
                .select((
                    account_id::uuid,
                    profile::profile_text.assume_not_null(),
                    profile_moderation::rejected_reason_category,
                    profile_moderation::rejected_reason_details,
                ))
                .load::<ProfileStringPendingModeration>(self.conn()),
        }
        .into_db_error(())?;

        Ok(ProfileStringModerationQueuePage { values })
    }
}
