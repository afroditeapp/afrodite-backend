use diesel::{
    Selectable,
    prelude::{AsChangeset, Insertable, Queryable},
    sql_types::BigInt,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_struct_try_from;
use simple_backend_utils::time::{TimeValue, UtcTimeValue};
use utoipa::ToSchema;

use crate::SelectedWeekdays;

/// Timestamp value in seconds which is
/// in inclusive range `[0, (SECONDS_IN_DAY - 1)]`.
///
/// This serializes to i64, so this must not be added to API doc.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Eq,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = BigInt)]
#[serde(try_from = "i64")]
#[serde(into = "i64")]
pub struct DayTimestamp {
    value: u32,
}

impl DayTimestamp {
    pub const MIN: u32 = 0;
    pub const MAX: u32 = (24 * 60 * 60) - 1;

    pub fn new_clamped(value: u32) -> Self {
        Self {
            value: value.clamp(Self::MIN, Self::MAX),
        }
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn from_hours(hours: u32) -> Self {
        Self::new_clamped(hours * 60 * 60)
    }

    pub fn to_utc_time_value(&self) -> UtcTimeValue {
        let minutes = self.value / 60;
        let hours = minutes / 60;
        let minutes_without_hours = minutes - (hours * 60);
        UtcTimeValue(TimeValue::new(hours as u8, minutes_without_hours as u8))
    }
}

impl Default for DayTimestamp {
    fn default() -> Self {
        Self { value: Self::MIN }
    }
}

impl TryFrom<i64> for DayTimestamp {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value < Self::MIN as i64 || value > Self::MAX as i64 {
            Err(format!(
                "DayTimestamp must be in range [{}, {}]",
                Self::MIN,
                Self::MAX
            ))
        } else {
            Ok(Self {
                value: value as u32,
            })
        }
    }
}

impl From<DayTimestamp> for i64 {
    fn from(value: DayTimestamp) -> Self {
        value.value as i64
    }
}

diesel_i64_struct_try_from!(DayTimestamp);

/// Timezone for timestamps is UTC+0.
#[derive(
    Debug,
    Clone,
    Copy,
    Queryable,
    Selectable,
    AsChangeset,
    Insertable,
    Deserialize,
    Serialize,
    ToSchema,
)]
#[diesel(table_name = crate::schema::admin_notification_settings)]
#[diesel(check_for_backend(crate::Db))]
pub struct AdminNotificationSettings {
    pub weekdays: SelectedWeekdays,
    #[schema(value_type = i64)]
    pub daily_enabled_time_start_seconds: DayTimestamp,
    #[schema(value_type = i64)]
    pub daily_enabled_time_end_seconds: DayTimestamp,
}

impl Default for AdminNotificationSettings {
    fn default() -> Self {
        Self {
            weekdays: SelectedWeekdays::all(),
            daily_enabled_time_start_seconds: DayTimestamp::new_clamped(DayTimestamp::MIN),
            daily_enabled_time_end_seconds: DayTimestamp::new_clamped(DayTimestamp::MAX),
        }
    }
}

/// Admin notification values or subscription info
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Deserialize,
    Serialize,
    Queryable,
    Selectable,
    AsChangeset,
    Insertable,
    ToSchema,
)]
#[diesel(table_name = crate::schema::admin_notification_subscriptions)]
#[diesel(check_for_backend(crate::Db))]
pub struct AdminNotification {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_initial_media_content_bot: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_initial_media_content_human: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_media_content_bot: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_media_content_human: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_profile_texts_bot: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_profile_texts_human: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_profile_names_bot: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_profile_names_human: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub process_reports: bool,
}

impl AdminNotification {
    pub fn enable(&mut self, event: AdminNotificationTypes) {
        match event {
            AdminNotificationTypes::ModerateInitialMediaContentBot => {
                self.moderate_initial_media_content_bot = true
            }
            AdminNotificationTypes::ModerateInitialMediaContentHuman => {
                self.moderate_initial_media_content_human = true
            }
            AdminNotificationTypes::ModerateMediaContentBot => {
                self.moderate_media_content_bot = true
            }
            AdminNotificationTypes::ModerateMediaContentHuman => {
                self.moderate_media_content_human = true
            }
            AdminNotificationTypes::ModerateProfileTextsBot => {
                self.moderate_profile_texts_bot = true
            }
            AdminNotificationTypes::ModerateProfileTextsHuman => {
                self.moderate_profile_texts_human = true
            }
            AdminNotificationTypes::ModerateProfileNamesBot => {
                self.moderate_profile_names_bot = true
            }
            AdminNotificationTypes::ModerateProfileNamesHuman => {
                self.moderate_profile_names_human = true
            }
            AdminNotificationTypes::ProcessReports => self.process_reports = true,
        }
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            moderate_initial_media_content_bot: self.moderate_initial_media_content_bot
                || other.moderate_initial_media_content_bot,
            moderate_initial_media_content_human: self.moderate_initial_media_content_human
                || other.moderate_initial_media_content_human,
            moderate_media_content_bot: self.moderate_media_content_bot
                || other.moderate_media_content_bot,
            moderate_media_content_human: self.moderate_media_content_human
                || other.moderate_media_content_human,
            moderate_profile_texts_bot: self.moderate_profile_texts_bot
                || other.moderate_profile_texts_bot,
            moderate_profile_texts_human: self.moderate_profile_texts_human
                || other.moderate_profile_texts_human,
            moderate_profile_names_bot: self.moderate_profile_names_bot
                || other.moderate_profile_names_bot,
            moderate_profile_names_human: self.moderate_profile_names_human
                || other.moderate_profile_names_human,
            process_reports: self.process_reports || other.process_reports,
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            moderate_initial_media_content_bot: self.moderate_initial_media_content_bot
                && other.moderate_initial_media_content_bot,
            moderate_initial_media_content_human: self.moderate_initial_media_content_human
                && other.moderate_initial_media_content_human,
            moderate_media_content_bot: self.moderate_media_content_bot
                && other.moderate_media_content_bot,
            moderate_media_content_human: self.moderate_media_content_human
                && other.moderate_media_content_human,
            moderate_profile_texts_bot: self.moderate_profile_texts_bot
                && other.moderate_profile_texts_bot,
            moderate_profile_texts_human: self.moderate_profile_texts_human
                && other.moderate_profile_texts_human,
            moderate_profile_names_bot: self.moderate_profile_names_bot
                && other.moderate_profile_names_bot,
            moderate_profile_names_human: self.moderate_profile_names_human
                && other.moderate_profile_names_human,
            process_reports: self.process_reports && other.process_reports,
        }
    }
}

pub enum AdminNotificationTypes {
    ModerateInitialMediaContentBot,
    ModerateInitialMediaContentHuman,
    ModerateMediaContentBot,
    ModerateMediaContentHuman,
    ModerateProfileTextsBot,
    ModerateProfileTextsHuman,
    ModerateProfileNamesBot,
    ModerateProfileNamesHuman,
    ProcessReports,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetAdminNotification {
    /// If true, client should not show the notification
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub hidden: bool,
    pub state: AdminNotification,
}
