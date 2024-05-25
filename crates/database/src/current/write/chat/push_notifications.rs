use diesel::{prelude::*, select, update};
use error_stack::Result;
use model::{AccountId, AccountIdInternal, FcmDeviceToken, PendingNotification};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

define_write_commands!(CurrentWriteChatPushNotifications, CurrentSyncWriteChatPushNotifications);

impl<C: ConnectionProvider> CurrentSyncWriteChatPushNotifications<C> {
    pub fn update_fcm_device_token(
        &mut self,
        id: AccountIdInternal,
        token: Option<FcmDeviceToken>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        // Remove the token from other accounts. It is possible that
        // same device is used for multiple accounts.
        update(chat_state.filter(fcm_device_token.eq(token.clone())))
            .set((
                fcm_device_token.eq(None::<FcmDeviceToken>),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        update(chat_state.find(id.as_db_id()))
            .set((
                fcm_device_token.eq(token),
                fcm_notification_sent.eq(false),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
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
            .set((
                pending_notification.eq(0),
                fcm_notification_sent.eq(false),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn get_and_reset_pending_notification_with_device_token(
        &mut self,
        token: FcmDeviceToken,
    ) -> Result<PendingNotification, DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        let token_clone = token.clone();
        let notification = chat_state
            .filter(fcm_device_token.eq(token_clone))
            .select(pending_notification)
            .first(self.conn())
            .into_db_error(())?;

        update(chat_state.filter(fcm_device_token.eq(token)))
            .set((
                pending_notification.eq(0),
                fcm_notification_sent.eq(false),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(notification)
    }

    pub fn enable_push_notification_sent_flag(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        update(chat_state.find(id.as_db_id()))
            .set((
                fcm_notification_sent.eq(true),
            ))
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

        let new_notification_value = notification | notification_to_be_added.value;

        let (token, notification_sent) = update(chat_state.find(id.as_db_id()))
            .set((
                pending_notification.eq(new_notification_value),
            ))
            .returning((fcm_device_token, fcm_notification_sent))
            .get_result(self.conn())
            .into_db_error(())?;

        Ok(PushNotificationStateInfo {
            fcm_device_token: token,
            fcm_notification_sent: notification_sent,
        })
    }
}

#[derive(Debug)]
pub struct PushNotificationStateInfo {
    pub fcm_device_token: Option<FcmDeviceToken>,
    pub fcm_notification_sent: bool,
}
