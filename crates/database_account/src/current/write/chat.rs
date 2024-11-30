use database::{define_current_write_commands, DieselDatabaseError, IntoDatabaseError};
use diesel::{prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, FcmDeviceToken, PendingNotificationToken};

define_current_write_commands!(CurrentWriteChatUtils);

impl CurrentWriteChatUtils<'_> {
    pub fn remove_fcm_device_token_and_pending_notification_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        update(chat_state.find(id.as_db_id()))
            .set((
                fcm_device_token.eq(None::<FcmDeviceToken>),
                fcm_notification_sent.eq(false),
                pending_notification_token.eq(None::<PendingNotificationToken>),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
