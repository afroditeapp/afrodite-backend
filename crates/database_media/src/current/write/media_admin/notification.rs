use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*, ExpressionMethods};
use error_stack::Result;
use model::AccountIdInternal;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteMediaAdminNotification);

impl CurrentWriteMediaAdminNotification<'_> {
    pub fn show_media_content_accepted_notification(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_app_notification_state::dsl::*;

        insert_into(media_app_notification_state)
            .values((
                account_id.eq(id.as_db_id()),
                media_content_accepted.eq(true),
            ))
            .on_conflict(account_id)
            .do_update()
            .set(media_content_accepted.eq(true))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn show_media_content_rejected_notification(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_app_notification_state::dsl::*;

        insert_into(media_app_notification_state)
            .values((
                account_id.eq(id.as_db_id()),
                media_content_rejected.eq(true),
            ))
            .on_conflict(account_id)
            .do_update()
            .set(media_content_rejected.eq(true))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
