
use diesel::{prelude::*, sql_types::Bool};

use model::{AccountIdInternal, AdminNotification};
use simple_backend_database::diesel_db::DieselDatabaseError;
use error_stack::{Result, ResultExt};

use crate::{define_current_read_commands, IntoDatabaseError};

define_current_read_commands!(CurrentReadAccountAdminNotification);

impl CurrentReadAccountAdminNotification<'_> {
    pub fn admin_notification_subscriptions(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AdminNotification, DieselDatabaseError> {
        use crate::schema::admin_notification_subscriptions::dsl::*;

        admin_notification_subscriptions
            .filter(account_id.eq(id.as_db_id()))
            .select(AdminNotification::as_select())
            .first(self.conn())
            .optional()
            .map(|v| v.unwrap_or_default())
            .change_context(DieselDatabaseError::Execute)
    }

    pub fn get_accounts_with_some_wanted_subscriptions(
        &mut self,
        wanted: AdminNotification,
    ) -> Result<Vec<(AccountIdInternal, AdminNotification)>, DieselDatabaseError> {
        use crate::schema::account_id;
        use crate::schema::admin_notification_subscriptions::dsl::*;

        admin_notification_subscriptions
            .inner_join(account_id::table)
            .filter(
                (moderate_media_content_bot.eq(true).and(wanted.moderate_media_content_bot.into_sql::<Bool>()))
                    .or(
                        moderate_media_content_human.eq(true).and(wanted.moderate_media_content_human.into_sql::<Bool>())
                    )
                    .or(
                        moderate_profile_texts_bot.eq(true).and(wanted.moderate_profile_texts_bot.into_sql::<Bool>())
                    )
                    .or(
                        moderate_profile_texts_human.eq(true).and(wanted.moderate_profile_texts_human.into_sql::<Bool>())
                    )
                    .or(
                        moderate_profile_names_bot.eq(true).and(wanted.moderate_profile_names_bot.into_sql::<Bool>())
                    )
                    .or(
                        moderate_profile_names_human.eq(true).and(wanted.moderate_profile_names_human.into_sql::<Bool>())
                    )
                    .or(
                        process_reports.eq(true).and(wanted.process_reports.into_sql::<Bool>())
                    )
            )
            .select((
                AccountIdInternal::as_select(),
                AdminNotification::as_select(),
            ))
            .load(self.conn())
            .into_db_error(())
    }
}
