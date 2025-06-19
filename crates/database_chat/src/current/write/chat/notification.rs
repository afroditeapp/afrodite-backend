use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{ExpressionMethods, insert_into, prelude::*};
use error_stack::Result;
use model::AccountIdInternal;
use model_chat::ChatAppNotificationSettings;

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
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
