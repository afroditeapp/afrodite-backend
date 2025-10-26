use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::{AccountIdInternal, AdminNotification, AdminNotificationSettings};
use simple_backend_database::diesel_db::DieselDatabaseError;
use simple_backend_utils::db::MyRunQueryDsl;

use crate::{IntoDatabaseError, define_current_write_commands};

define_current_write_commands!(CurrentWriteCommonAdminNotification);

impl CurrentWriteCommonAdminNotification<'_> {
    pub fn set_admin_notification_settings(
        &mut self,
        id: AccountIdInternal,
        settings: AdminNotificationSettings,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::admin_notification_settings::dsl::*;

        insert_into(admin_notification_settings)
            .values((account_id.eq(id.as_db_id()), &settings))
            .on_conflict(account_id)
            .do_update()
            .set(&settings)
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn set_admin_notification_subscriptions(
        &mut self,
        id: AccountIdInternal,
        subscriptions: AdminNotification,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::admin_notification_subscriptions::dsl::*;

        insert_into(admin_notification_subscriptions)
            .values((account_id.eq(id.as_db_id()), &subscriptions))
            .on_conflict(account_id)
            .do_update()
            .set(&subscriptions)
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
