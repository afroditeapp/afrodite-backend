use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::AccountIdInternal;
use model_account::AccountAppNotificationSettings;

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountNotification);

impl CurrentReadAccountNotification<'_> {
    pub fn app_notification_settings(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AccountAppNotificationSettings, DieselDatabaseError> {
        use crate::schema::account_app_notification_settings::dsl::*;

        let query_result = account_app_notification_settings
            .filter(account_id.eq(id.as_db_id()))
            .select(AccountAppNotificationSettings::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(query_result.unwrap_or_default())
    }
}
