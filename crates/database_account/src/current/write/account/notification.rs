use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{ExpressionMethods, insert_into, prelude::*};
use error_stack::Result;
use model::AccountIdInternal;
use model_account::AccountAppNotificationSettings;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountNotification);

impl CurrentWriteAccountNotification<'_> {
    pub fn upsert_app_notification_settings(
        &mut self,
        id: AccountIdInternal,
        settings: AccountAppNotificationSettings,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_app_notification_settings::dsl::*;

        insert_into(account_app_notification_settings)
            .values((account_id.eq(id.as_db_id()), settings))
            .on_conflict(account_id)
            .do_update()
            .set(settings)
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
