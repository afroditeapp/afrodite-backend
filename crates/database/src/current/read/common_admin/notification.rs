
use diesel::prelude::*;

use model::{AccountIdInternal, AdminNotificationSubscriptions};
use simple_backend_database::diesel_db::DieselDatabaseError;
use error_stack::{Result, ResultExt};

use crate::define_current_read_commands;

define_current_read_commands!(CurrentReadAccountAdminNotification);

impl CurrentReadAccountAdminNotification<'_> {
    pub fn admin_notification_subscriptions(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AdminNotificationSubscriptions, DieselDatabaseError> {
        use crate::schema::admin_notification_subscriptions::dsl::*;

        admin_notification_subscriptions
            .filter(account_id.eq(id.as_db_id()))
            .select(AdminNotificationSubscriptions::as_select())
            .first(self.conn())
            .optional()
            .map(|v| v.unwrap_or_default())
            .change_context(DieselDatabaseError::Execute)
    }
}
