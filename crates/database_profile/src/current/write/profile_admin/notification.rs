use std::num::Wrapping;

use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{ExpressionMethods, insert_into, prelude::*};
use error_stack::Result;
use model::AccountIdInternal;

use crate::{IntoDatabaseError, current::read::GetDbReadCommandsProfile};

define_current_write_commands!(CurrentWriteProfileAdminNotification);

impl CurrentWriteProfileAdminNotification<'_> {
    pub fn show_profile_text_accepted_notification(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_app_notification_state::dsl::*;

        let current = self
            .read()
            .profile()
            .notification()
            .profile_text_moderation_completed(id)?;

        let new_value = Wrapping(current.accepted) + Wrapping(1);
        let new_value: i64 = new_value.0.into();

        insert_into(profile_app_notification_state)
            .values((
                account_id.eq(id.as_db_id()),
                profile_text_accepted.eq(new_value),
            ))
            .on_conflict(account_id)
            .do_update()
            .set(profile_text_accepted.eq(new_value))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn show_profile_text_rejected_notification(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_app_notification_state::dsl::*;

        let current = self
            .read()
            .profile()
            .notification()
            .profile_text_moderation_completed(id)?;

        let new_value = Wrapping(current.rejected) + Wrapping(1);
        let new_value: i64 = new_value.0.into();

        insert_into(profile_app_notification_state)
            .values((
                account_id.eq(id.as_db_id()),
                profile_text_rejected.eq(new_value),
            ))
            .on_conflict(account_id)
            .do_update()
            .set(profile_text_rejected.eq(new_value))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
