use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, FcmDeviceToken, PendingNotification, PendingNotificationToken, PushNotificationStateInfo};

use crate::IntoDatabaseError;

define_current_write_commands!(
    CurrentWriteChatPushNotifications
);

impl CurrentWriteChatPushNotifications<'_> {
    pub fn remove_fcm_device_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        update(chat_state.find(id.as_db_id()))
            .set((
                fcm_device_token.eq(None::<FcmDeviceToken>),
                fcm_notification_sent.eq(false),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn update_fcm_device_token_and_generate_new_notification_token(
        &mut self,
        id: AccountIdInternal,
        token: FcmDeviceToken,
    ) -> Result<PendingNotificationToken, DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        // Remove the token from other accounts. It is possible that
        // same device is used for multiple accounts.
        update(chat_state.filter(fcm_device_token.eq(token.clone())))
            .set(fcm_device_token.eq(None::<FcmDeviceToken>))
            .execute(self.conn())
            .into_db_error(())?;

        let notification_token = PendingNotificationToken::generate_new();

        update(chat_state.find(id.as_db_id()))
            .set((
                fcm_device_token.eq(token),
                fcm_notification_sent.eq(false),
                pending_notification_token.eq(notification_token.clone()),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(notification_token)
    }

    pub fn update_fcm_notification_sent_value(
        &mut self,
        id: AccountIdInternal,
        fcm_notification_sent_value: bool,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        update(chat_state.find(id.as_db_id()))
            .set(fcm_notification_sent.eq(fcm_notification_sent_value))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn reset_pending_notification(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        update(chat_state.find(id.as_db_id()))
            .set((pending_notification.eq(0), fcm_notification_sent.eq(false)))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn get_and_reset_pending_notification_with_notification_token(
        &mut self,
        token: PendingNotificationToken,
    ) -> Result<(AccountIdInternal, PendingNotification), DieselDatabaseError> {
        use model::schema::{chat_state, account_id};

        let token_clone = token.clone();
        let (id, notification) = chat_state::table
            .inner_join(account_id::table)
            .filter(chat_state::pending_notification_token.eq(token_clone))
            .select((AccountIdInternal::as_select(), chat_state::pending_notification))
            .first(self.conn())
            .into_db_error(())?;

        update(chat_state::table.filter(chat_state::pending_notification_token.eq(token)))
            .set((chat_state::pending_notification.eq(0), chat_state::fcm_notification_sent.eq(false)))
            .execute(self.conn())
            .into_db_error(())?;

        Ok((id, notification))
    }

    pub fn enable_push_notification_sent_flag(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        update(chat_state.find(id.as_db_id()))
            .set((fcm_notification_sent.eq(true),))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn get_push_notification_state_info_and_add_notification_value(
        &mut self,
        id: AccountIdInternal,
        notification_to_be_added: PendingNotification,
    ) -> Result<PushNotificationStateInfo, DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        let notification: i64 = chat_state
            .filter(account_id.eq(id.as_db_id()))
            .select(pending_notification)
            .first(self.conn())
            .into_db_error(())?;

        let new_notification_value = notification | *notification_to_be_added.as_i64();

        let (token, notification_sent) = update(chat_state.find(id.as_db_id()))
            .set((pending_notification.eq(new_notification_value),))
            .returning((fcm_device_token, fcm_notification_sent))
            .get_result(self.conn())
            .into_db_error(())?;

        Ok(PushNotificationStateInfo {
            fcm_device_token: token,
            fcm_notification_sent: notification_sent,
        })
    }
}
