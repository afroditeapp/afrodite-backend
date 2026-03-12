use diesel::{delete, insert_into, prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, PendingAppNotification, PendingAppNotificationType};
use simple_backend_utils::db::MyRunQueryDsl;

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_write_commands};

define_current_write_commands!(CurrentWriteCommonNotification);

impl CurrentWriteCommonNotification<'_> {
    pub fn upsert_pending_app_notification(
        &mut self,
        id: AccountIdInternal,
        type_number: PendingAppNotificationType,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_app_notifications::dsl::*;

        insert_into(pending_app_notifications)
            .values((
                account_id.eq(id.as_db_id()),
                notification_type_number.eq(type_number),
                push_notification_sent.eq(false),
            ))
            .on_conflict((account_id, notification_type_number))
            .do_update()
            .set(push_notification_sent.eq(false))
            .execute_my_conn(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn mark_pending_app_notifications_push_sent(
        &mut self,
        id: AccountIdInternal,
        type_numbers: Vec<PendingAppNotificationType>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_app_notifications::dsl::*;

        if type_numbers.is_empty() {
            return Ok(());
        }

        update(pending_app_notifications)
            .filter(account_id.eq(id.as_db_id()))
            .filter(notification_type_number.eq_any(type_numbers))
            .set(push_notification_sent.eq(true))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn delete_pending_app_notifications(
        &mut self,
        id: AccountIdInternal,
        notifications: Vec<PendingAppNotification>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_app_notifications::dsl::*;

        if notifications.is_empty() {
            return Ok(());
        }

        let mut push_sent_type_numbers = Vec::new();
        let mut push_not_sent_type_numbers = Vec::new();

        for notification in notifications {
            if notification.push_notification_sent {
                push_sent_type_numbers.push(notification.notification_type);
            } else {
                push_not_sent_type_numbers.push(notification.notification_type);
            }
        }

        if !push_sent_type_numbers.is_empty() {
            delete(pending_app_notifications)
                .filter(account_id.eq(id.as_db_id()))
                .filter(notification_type_number.eq_any(push_sent_type_numbers))
                .filter(push_notification_sent.eq(true))
                .execute(self.conn())
                .into_db_error(id)?;
        }

        if !push_not_sent_type_numbers.is_empty() {
            delete(pending_app_notifications)
                .filter(account_id.eq(id.as_db_id()))
                .filter(notification_type_number.eq_any(push_not_sent_type_numbers))
                .filter(push_notification_sent.eq(false))
                .execute(self.conn())
                .into_db_error(id)?;
        }

        Ok(())
    }
}
