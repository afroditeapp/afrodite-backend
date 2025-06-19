use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model_chat::{AccountIdInternal, ChatAppNotificationSettings};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadChatNotification);

impl CurrentReadChatNotification<'_> {
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
}
