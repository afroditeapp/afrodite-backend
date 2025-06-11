
use diesel::{insert_into, prelude::*};

use model::{AccountIdInternal, AdminNotification};
use simple_backend_database::diesel_db::DieselDatabaseError;
use error_stack::Result;

use crate::{define_current_write_commands, IntoDatabaseError};

define_current_write_commands!(CurrentWriteCommonAdminNotification);

impl CurrentWriteCommonAdminNotification<'_> {
    pub fn set_admin_notification_subscriptions(
        &mut self,
        id: AccountIdInternal,
        subscriptions: AdminNotification,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::admin_notification_subscriptions::dsl::*;

        insert_into(admin_notification_subscriptions)
            .values((
                account_id.eq(id.as_db_id()),
                &subscriptions,
            ))
            .on_conflict(account_id)
            .do_update()
            .set(&subscriptions)
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
