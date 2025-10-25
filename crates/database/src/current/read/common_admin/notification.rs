use chrono::{Datelike, Timelike, Utc};
use diesel::{
    prelude::*,
    sql_types::{Bool, Integer},
};
use error_stack::{Result, ResultExt};
use model::{
    AccountIdInternal, AdminNotification, AdminNotificationSettings, DayTimestamp,
    SelectedWeekdays, WeekdayFlags,
};
use simple_backend_database::diesel_db::DieselDatabaseError;

use crate::{IntoDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentReadAccountAdminNotification);

impl CurrentReadAccountAdminNotification<'_> {
    pub fn admin_notification_settings(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AdminNotificationSettings, DieselDatabaseError> {
        use crate::schema::admin_notification_settings::dsl::*;

        admin_notification_settings
            .filter(account_id.eq(id.as_db_id()))
            .select(AdminNotificationSettings::as_select())
            .first(self.conn())
            .optional()
            .map(|v| v.unwrap_or_default())
            .change_context(DieselDatabaseError::Execute)
    }

    pub fn nearest_start_time(&mut self) -> Result<DayTimestamp, DieselDatabaseError> {
        use crate::schema::admin_notification_settings::dsl::*;

        let current_time = Utc::now();
        let current_hour = TryInto::<u8>::try_into(current_time.hour())
            .change_context(DieselDatabaseError::DataFormatConversion)?;
        let day_timestamp = DayTimestamp::from_hours(current_hour);

        let current_day = admin_notification_settings
            .filter(daily_enabled_time_start_seconds.gt(day_timestamp))
            .select(daily_enabled_time_start_seconds)
            .order(daily_enabled_time_start_seconds.asc())
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)?;

        let next_day = admin_notification_settings
            .select(daily_enabled_time_start_seconds)
            .order(daily_enabled_time_start_seconds.asc())
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)?;

        Ok(current_day
            .or(next_day)
            .unwrap_or(DayTimestamp::from_hours(0)))
    }

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

    pub fn get_accounts_which_should_receive_notification(
        &mut self,
        wanted: AdminNotification,
    ) -> Result<Vec<(AccountIdInternal, AdminNotification)>, DieselDatabaseError> {
        use crate::schema::{
            account_id, admin_notification_settings, admin_notification_subscriptions::dsl::*,
        };

        let current_time = Utc::now();
        let current_weekday: WeekdayFlags = current_time.weekday().into();
        let current_hour = TryInto::<u8>::try_into(current_time.hour())
            .change_context(DieselDatabaseError::DataFormatConversion)?;
        let day_timestamp = DayTimestamp::from_hours(current_hour);

        let data: Vec<(
            AccountIdInternal,
            AdminNotification,
            Option<SelectedWeekdays>,
        )> = admin_notification_subscriptions
            .inner_join(account_id::table)
            .left_join(
                admin_notification_settings::table
                    .on(admin_notification_settings::account_id.eq(account_id::id)),
            )
            // Bitwise AND does not work, so do weekday filtering manually
            // .filter(admin_notification_settings::weekdays.is_null()
            //     .or((admin_notification_settings::weekdays & current_weekday.as_sql::<BigInt>()).ne(0))
            // )
            .filter(
                admin_notification_settings::daily_enabled_time_start_seconds
                    .is_null()
                    .or(day_timestamp.as_sql::<Integer>().between(
                        admin_notification_settings::daily_enabled_time_start_seconds,
                        admin_notification_settings::daily_enabled_time_end_seconds,
                    )),
            )
            .filter(
                (moderate_media_content_bot
                    .eq(true)
                    .and(wanted.moderate_media_content_bot.into_sql::<Bool>()))
                .or(moderate_media_content_human
                    .eq(true)
                    .and(wanted.moderate_media_content_human.into_sql::<Bool>()))
                .or(moderate_profile_texts_bot
                    .eq(true)
                    .and(wanted.moderate_profile_texts_bot.into_sql::<Bool>()))
                .or(moderate_profile_texts_human
                    .eq(true)
                    .and(wanted.moderate_profile_texts_human.into_sql::<Bool>()))
                .or(moderate_profile_names_bot
                    .eq(true)
                    .and(wanted.moderate_profile_names_bot.into_sql::<Bool>()))
                .or(moderate_profile_names_human
                    .eq(true)
                    .and(wanted.moderate_profile_names_human.into_sql::<Bool>()))
                .or(process_reports
                    .eq(true)
                    .and(wanted.process_reports.into_sql::<Bool>())),
            )
            .select((
                AccountIdInternal::as_select(),
                AdminNotification::as_select(),
                admin_notification_settings::weekdays.nullable(),
            ))
            .load(self.conn())
            .into_db_error(())?;

        let data = data
            .into_iter()
            .filter_map(|(id, notification, selected_weekdays)| {
                if let Some(selected_weekdays) = selected_weekdays {
                    let selected_weekdays_flags: WeekdayFlags = selected_weekdays.into();
                    if !selected_weekdays_flags.contains(current_weekday) {
                        return None;
                    }
                }
                Some((id, notification))
            })
            .collect();

        Ok(data)
    }
}
