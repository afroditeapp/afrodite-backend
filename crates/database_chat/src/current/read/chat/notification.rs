use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::{NewMessagePushNotification, NewMessagePushNotificationList, UnixTime};
use model_chat::{
    AccountIdInternal, ChatAppNotificationSettings, ChatEmailNotificationSettings,
    PendingChatNotification,
};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadChatNotification);

impl CurrentReadChatNotification<'_> {
    pub fn new_message_notification_list(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<NewMessagePushNotificationList, DieselDatabaseError> {
        use crate::schema::{account_id, conversation_id, pending_chat_notifications};

        let data: Vec<(AccountIdInternal, model::ConversationId, i64)> =
            pending_chat_notifications::table
                .inner_join(
                    conversation_id::table.on(conversation_id::account_id
                        .eq(pending_chat_notifications::account_id_viewer)
                        .and(
                            conversation_id::other_account_id
                                .eq(pending_chat_notifications::account_id_sender),
                        )),
                )
                .inner_join(
                    account_id::table
                        .on(pending_chat_notifications::account_id_sender.eq(account_id::id)),
                )
                .filter(
                    pending_chat_notifications::account_id_viewer.eq(account_id_value.as_db_id()),
                )
                .filter(pending_chat_notifications::push_notification_sent.eq(false))
                .select((
                    AccountIdInternal::as_select(),
                    conversation_id::id,
                    pending_chat_notifications::message_count,
                ))
                .order_by(conversation_id::id.asc())
                .load(self.conn())
                .into_db_error(())?;

        Ok(NewMessagePushNotificationList {
            values: data
                .into_iter()
                .map(|(message_sender, conversation_id, message_count)| {
                    NewMessagePushNotification {
                        message_sender,
                        conversation_id,
                        message_count,
                    }
                })
                .collect(),
        })
    }

    pub fn app_notification_settings(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<ChatAppNotificationSettings, DieselDatabaseError> {
        use crate::schema::chat_app_notification_settings::dsl::*;

        let query_result = chat_app_notification_settings
            .filter(account_id.eq(account_id_value.as_db_id()))
            .select(ChatAppNotificationSettings::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(query_result.unwrap_or_default())
    }

    pub fn messages_without_sent_email_notification(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<Vec<UnixTime>, DieselDatabaseError> {
        use crate::schema::pending_chat_notifications;

        pending_chat_notifications::table
            .filter(pending_chat_notifications::account_id_viewer.eq(account_id_value.as_db_id()))
            .filter(pending_chat_notifications::email_notification_sent.eq(false))
            .select(pending_chat_notifications::created_unix_time)
            .load(self.conn())
            .into_db_error(())
    }

    pub fn email_notification_settings(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<ChatEmailNotificationSettings, DieselDatabaseError> {
        use crate::schema::chat_email_notification_settings::dsl::*;

        let query_result = chat_email_notification_settings
            .filter(account_id.eq(account_id_value.as_db_id()))
            .select(ChatEmailNotificationSettings::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(query_result.unwrap_or_default())
    }

    pub fn pending_chat_notifications(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<Vec<PendingChatNotification>, DieselDatabaseError> {
        use crate::schema::{account_id, pending_chat_notifications};

        let data: Vec<(model::AccountId, i64, bool)> = pending_chat_notifications::table
            .inner_join(
                account_id::table
                    .on(pending_chat_notifications::account_id_sender.eq(account_id::id)),
            )
            .filter(pending_chat_notifications::account_id_viewer.eq(account_id_value.as_db_id()))
            .select((
                account_id::uuid,
                pending_chat_notifications::message_count,
                pending_chat_notifications::push_notification_sent,
            ))
            .order_by(pending_chat_notifications::account_id_sender.asc())
            .load(self.conn())
            .into_db_error(())?;

        Ok(data
            .into_iter()
            .map(
                |(account_id_sender, message_count, push_notification_sent)| {
                    PendingChatNotification {
                        account_id_sender,
                        message_count,
                        push_notification_sent,
                    }
                },
            )
            .collect())
    }
}
