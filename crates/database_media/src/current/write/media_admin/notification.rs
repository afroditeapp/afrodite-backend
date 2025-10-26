use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::AccountIdInternal;
use simple_backend_utils::db::MyRunQueryDsl;

use crate::{IntoDatabaseError, current::read::GetDbReadCommandsMedia};

define_current_write_commands!(CurrentWriteMediaAdminNotification);

impl CurrentWriteMediaAdminNotification<'_> {
    pub fn show_media_content_accepted_notification(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_app_notification_state::dsl::*;

        let current = self
            .read()
            .media()
            .notification()
            .media_content_moderation_completed(id)?;

        let new_value = current.accepted.id.wrapping_increment();

        insert_into(media_app_notification_state)
            .values((
                account_id.eq(id.as_db_id()),
                media_content_accepted.eq(new_value),
            ))
            .on_conflict(account_id)
            .do_update()
            .set(media_content_accepted.eq(new_value))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn show_media_content_rejected_notification(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_app_notification_state::dsl::*;

        let current = self
            .read()
            .media()
            .notification()
            .media_content_moderation_completed(id)?;

        let new_value = current.rejected.id.wrapping_increment();

        insert_into(media_app_notification_state)
            .values((
                account_id.eq(id.as_db_id()),
                media_content_rejected.eq(new_value),
            ))
            .on_conflict(account_id)
            .do_update()
            .set(media_content_rejected.eq(new_value))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn show_media_content_deleted_notification(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_app_notification_state::dsl::*;

        let current = self
            .read()
            .media()
            .notification()
            .media_content_moderation_completed(id)?;

        let new_value = current.deleted.id.wrapping_increment();

        insert_into(media_app_notification_state)
            .values((
                account_id.eq(id.as_db_id()),
                media_content_deleted.eq(new_value),
            ))
            .on_conflict(account_id)
            .do_update()
            .set(media_content_deleted.eq(new_value))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
