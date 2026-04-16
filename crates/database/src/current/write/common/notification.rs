use diesel::{delete, insert_into, prelude::*, update};
use error_stack::Result;
use model::{
    AccountIdInternal, PendingAppNotification, PendingAppNotificationInternal,
    PendingAppNotificationToDelete, UnixTime,
};
use simple_backend_utils::db::MyRunQueryDsl;

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_write_commands};

define_current_write_commands!(CurrentWriteCommonNotification);

impl CurrentWriteCommonNotification<'_> {
    pub fn upsert_pending_app_notification(
        &mut self,
        id: AccountIdInternal,
        notification: PendingAppNotificationInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_app_notifications::dsl::*;

        let (type_number, data_integer_value) = notification.into_db_values();
        let current_time = UnixTime::current_time();

        insert_into(pending_app_notifications)
            .values((
                account_id.eq(id.as_db_id()),
                notification_type_number.eq(type_number),
                push_notification_sent.eq(false),
                email_notification_sent.eq(false),
                created_unix_time.eq(current_time),
                data_integer.eq(data_integer_value),
            ))
            .on_conflict((account_id, notification_type_number))
            .do_update()
            .set((
                push_notification_sent.eq(false),
                data_integer.eq(data_integer_value),
            ))
            .execute_my_conn(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn mark_pending_app_notifications_push_sent(
        &mut self,
        id: AccountIdInternal,
        notifications: Vec<PendingAppNotification>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_app_notifications::dsl::*;

        if notifications.is_empty() {
            return Ok(());
        }

        for notification in notifications {
            update(pending_app_notifications)
                .filter(account_id.eq(id.as_db_id()))
                .filter(notification_type_number.eq(notification.notification_type))
                .filter(push_notification_sent.eq(notification.push_notification_sent))
                .filter(crate::eq_optional!(data_integer, notification.data_integer))
                .set(push_notification_sent.eq(true))
                .execute(self.conn())
                .into_db_error(id)?;
        }

        Ok(())
    }

    pub fn mark_pending_app_notification_email_sent(
        &mut self,
        id: AccountIdInternal,
        notification: model::PendingAppNotificationType,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_app_notifications::dsl::*;

        update(pending_app_notifications)
            .filter(account_id.eq(id.as_db_id()))
            .filter(notification_type_number.eq(notification))
            .filter(email_notification_sent.eq(false))
            .set(email_notification_sent.eq(true))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn delete_pending_app_notifications(
        &mut self,
        id: AccountIdInternal,
        notifications: Vec<PendingAppNotificationToDelete>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_app_notifications::dsl::*;

        if notifications.is_empty() {
            return Ok(());
        }

        for notification in notifications {
            delete(pending_app_notifications)
                .filter(account_id.eq(id.as_db_id()))
                .filter(notification_type_number.eq(notification.notification_type))
                .filter(crate::eq_optional!(data_integer, notification.data_integer))
                .execute(self.conn())
                .into_db_error(id)?;
        }

        Ok(())
    }
}
