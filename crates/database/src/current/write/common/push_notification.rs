use diesel::{prelude::*, update};
use error_stack::Result;
use model::{
    AccountIdInternal, FcmDeviceToken, PendingNotification, PendingNotificationToken,
    PushNotificationStateInfo, PushNotificationType, UnixTime,
};

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentWriteCommonPushNotification);

impl CurrentWriteCommonPushNotification<'_> {
    pub fn remove_fcm_device_token_and_pending_notification_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        update(common_state.find(id.as_db_id()))
            .set((
                fcm_device_token.eq(None::<FcmDeviceToken>),
                fcm_device_token_unix_time.eq(None::<UnixTime>),
                fcm_data_notification_sent.eq(false),
                fcm_visible_notification_sent.eq(false),
                pending_notification_token.eq(None::<PendingNotificationToken>),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn remove_fcm_device_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        update(common_state.find(id.as_db_id()))
            .set((
                fcm_device_token.eq(None::<FcmDeviceToken>),
                fcm_device_token_unix_time.eq(None::<UnixTime>),
                fcm_data_notification_sent.eq(false),
                fcm_visible_notification_sent.eq(false),
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
        use model::schema::common_state::dsl::*;

        // Remove the token from other accounts. It is possible that
        // same device is used for multiple accounts.
        update(common_state.filter(fcm_device_token.eq(token.clone())))
            .set((
                fcm_device_token.eq(None::<FcmDeviceToken>),
                fcm_device_token_unix_time.eq(None::<UnixTime>),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        let notification_token = PendingNotificationToken::generate_new();

        update(common_state.find(id.as_db_id()))
            .set((
                fcm_device_token.eq(token),
                fcm_device_token_unix_time.eq(UnixTime::current_time()),
                fcm_data_notification_sent.eq(false),
                fcm_visible_notification_sent.eq(false),
                pending_notification_token.eq(notification_token.clone()),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(notification_token)
    }

    pub fn reset_pending_notification(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        update(common_state.find(id.as_db_id()))
            .set((
                pending_notification.eq(0),
                fcm_data_notification_sent.eq(false),
                fcm_visible_notification_sent.eq(false),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn get_and_reset_pending_notification_with_notification_token(
        &mut self,
        token: PendingNotificationToken,
    ) -> Result<(AccountIdInternal, PendingNotification), DieselDatabaseError> {
        use model::schema::{account_id, common_state};

        let token_clone = token.clone();
        let (id, notification) = common_state::table
            .inner_join(account_id::table)
            .filter(common_state::pending_notification_token.eq(token_clone))
            .select((
                AccountIdInternal::as_select(),
                common_state::pending_notification,
            ))
            .first(self.conn())
            .into_db_error(())?;

        update(common_state::table.filter(common_state::pending_notification_token.eq(token)))
            .set((
                common_state::pending_notification.eq(0),
                common_state::fcm_data_notification_sent.eq(false),
                common_state::fcm_visible_notification_sent.eq(false),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        Ok((id, notification))
    }

    pub fn enable_push_notification_sent_flag(
        &mut self,
        id: AccountIdInternal,
        notification: PushNotificationType,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        match notification {
            PushNotificationType::Data => {
                update(common_state.find(id.as_db_id()))
                    .set(fcm_data_notification_sent.eq(true))
                    .execute(self.conn())
                    .into_db_error(())?;
            }
            PushNotificationType::Visible => {
                update(common_state.find(id.as_db_id()))
                    .set(fcm_visible_notification_sent.eq(true))
                    .execute(self.conn())
                    .into_db_error(())?;
            }
        }

        Ok(())
    }

    pub fn get_push_notification_state_info_and_add_notification_value(
        &mut self,
        id: AccountIdInternal,
        notification_to_be_added: PendingNotification,
    ) -> Result<PushNotificationStateInfo, DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        let notification: i64 = common_state
            .filter(account_id.eq(id.as_db_id()))
            .select(pending_notification)
            .first(self.conn())
            .into_db_error(())?;

        let new_notification_value = notification | *notification_to_be_added.as_i64();

        let (token, data_notification_sent, visible_notification_sent) =
            update(common_state.find(id.as_db_id()))
                .set((pending_notification.eq(new_notification_value),))
                .returning((
                    fcm_device_token,
                    fcm_data_notification_sent,
                    fcm_visible_notification_sent,
                ))
                .get_result(self.conn())
                .into_db_error(())?;

        Ok(PushNotificationStateInfo {
            fcm_device_token: token,
            fcm_data_notification_sent: data_notification_sent,
            fcm_visible_notification_sent: visible_notification_sent,
        })
    }
}
