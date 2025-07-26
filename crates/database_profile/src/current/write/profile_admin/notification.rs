use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{ExpressionMethods, insert_into, prelude::*, upsert::excluded};
use error_stack::Result;
use model::AccountIdInternal;

use crate::{IntoDatabaseError, current::read::GetDbReadCommandsProfile};

define_current_write_commands!(CurrentWriteProfileAdminNotification);

impl CurrentWriteProfileAdminNotification<'_> {
    pub fn show_profile_name_moderation_completed_notification(
        &mut self,
        id: AccountIdInternal,
        accepted: bool,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_app_notification_state::dsl::*;

        let current = self
            .read()
            .profile()
            .notification()
            .profile_string_moderation_completed(id)?;

        let accepted_new_value: i64 = if accepted {
            current.name_accepted.id.wrapping_increment().id.into()
        } else {
            current.name_accepted.id.id.into()
        };

        let rejected_new_value: i64 = if !accepted {
            current.name_rejected.id.wrapping_increment().id.into()
        } else {
            current.name_rejected.id.id.into()
        };

        insert_into(profile_app_notification_state)
            .values((
                account_id.eq(id.as_db_id()),
                profile_name_accepted.eq(accepted_new_value),
                profile_name_rejected.eq(rejected_new_value),
            ))
            .on_conflict(account_id)
            .do_update()
            .set((
                profile_name_accepted.eq(excluded(profile_name_accepted)),
                profile_name_rejected.eq(excluded(profile_name_rejected)),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn show_profile_text_moderation_completed_notification(
        &mut self,
        id: AccountIdInternal,
        accepted: bool,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_app_notification_state::dsl::*;

        let current = self
            .read()
            .profile()
            .notification()
            .profile_string_moderation_completed(id)?;

        let accepted_new_value: i64 = if accepted {
            current.text_accepted.id.wrapping_increment().id.into()
        } else {
            current.text_accepted.id.id.into()
        };

        let rejected_new_value: i64 = if !accepted {
            current.text_rejected.id.wrapping_increment().id.into()
        } else {
            current.text_rejected.id.id.into()
        };

        insert_into(profile_app_notification_state)
            .values((
                account_id.eq(id.as_db_id()),
                profile_text_accepted.eq(accepted_new_value),
                profile_text_rejected.eq(rejected_new_value),
            ))
            .on_conflict(account_id)
            .do_update()
            .set((
                profile_text_accepted.eq(excluded(profile_text_accepted)),
                profile_text_rejected.eq(excluded(profile_text_rejected)),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
