use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{delete, insert_into, prelude::*, update};
use error_stack::Result;
use model::{AccountIdDb, AccountIdInternal, NewMessagePushNotification, UnixTime};
use model_chat::{
    ChatAppNotificationSettings, ChatEmailNotificationSettings, PendingChatNotification,
};
use simple_backend_utils::db::MyRunQueryDsl;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteChatNotification);

impl CurrentWriteChatNotification<'_> {
    pub fn upsert_app_notification_settings(
        &mut self,
        id: AccountIdInternal,
        settings: ChatAppNotificationSettings,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::chat_app_notification_settings::dsl::*;

        insert_into(chat_app_notification_settings)
            .values((account_id.eq(id.as_db_id()), settings))
            .on_conflict(account_id)
            .do_update()
            .set(settings)
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn upsert_email_notification_settings(
        &mut self,
        id: AccountIdInternal,
        settings: ChatEmailNotificationSettings,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::chat_email_notification_settings::dsl::*;

        insert_into(chat_email_notification_settings)
            .values((account_id.eq(id.as_db_id()), settings))
            .on_conflict(account_id)
            .do_update()
            .set(settings)
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn upsert_pending_chat_notification(
        &mut self,
        viewer_id: AccountIdInternal,
        sender_id: AccountIdInternal,
        message_count_value: i64,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_chat_notifications::dsl::*;

        let current_time = UnixTime::current_time();

        insert_into(pending_chat_notifications)
            .values((
                account_id_viewer.eq(viewer_id.as_db_id()),
                account_id_sender.eq(sender_id.as_db_id()),
                message_count.eq(message_count_value),
                created_unix_time.eq(current_time),
                email_notification_sent.eq(false),
                push_notification_sent.eq(false),
            ))
            .on_conflict((account_id_viewer, account_id_sender))
            .do_update()
            .set((
                message_count.eq(message_count_value),
                push_notification_sent.eq(false),
            ))
            .execute_my_conn(self.conn())
            .into_db_error(viewer_id)?;

        Ok(())
    }

    pub fn mark_pending_chat_notifications_push_sent(
        &mut self,
        viewer_id: AccountIdInternal,
        notifications: Vec<NewMessagePushNotification>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_chat_notifications::dsl::*;

        if notifications.is_empty() {
            return Ok(());
        }

        for notification in notifications {
            update(pending_chat_notifications)
                .filter(account_id_viewer.eq(viewer_id.as_db_id()))
                .filter(account_id_sender.eq(notification.message_sender.as_db_id()))
                .filter(message_count.eq(notification.message_count))
                .filter(push_notification_sent.eq(false))
                .set(push_notification_sent.eq(true))
                .execute(self.conn())
                .into_db_error(viewer_id)?;
        }

        Ok(())
    }

    pub fn mark_message_email_notification_sent(
        &mut self,
        viewer_id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_chat_notifications::dsl::*;

        update(pending_chat_notifications)
            .filter(account_id_viewer.eq(viewer_id.as_db_id()))
            .filter(email_notification_sent.eq(false))
            .set(email_notification_sent.eq(true))
            .execute(self.conn())
            .into_db_error(viewer_id)?;

        Ok(())
    }

    pub fn delete_pending_chat_notifications(
        &mut self,
        viewer_id: AccountIdInternal,
        notifications: Vec<PendingChatNotification>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::{account_id, pending_chat_notifications::dsl::*};

        if notifications.is_empty() {
            return Ok(());
        }

        for notification in notifications {
            let sender_db_id: Option<AccountIdDb> = account_id::table
                .filter(account_id::uuid.eq(notification.account_id_sender))
                .select(account_id::id)
                .first(self.conn())
                .optional()
                .into_db_error(viewer_id)?;

            let Some(sender_db_id) = sender_db_id else {
                continue;
            };

            delete(pending_chat_notifications)
                .filter(account_id_viewer.eq(viewer_id.as_db_id()))
                .filter(account_id_sender.eq(sender_db_id))
                .filter(message_count.eq(notification.message_count))
                .filter(push_notification_sent.eq(notification.push_notification_sent))
                .execute(self.conn())
                .into_db_error(viewer_id)?;
        }

        Ok(())
    }
}
