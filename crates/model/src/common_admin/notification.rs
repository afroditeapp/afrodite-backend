use diesel::{
    Selectable,
    prelude::{AsChangeset, Insertable, Queryable},
    sql_types::Integer,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i32_wrapper;
use simple_backend_utils::time::{TimeValue, UtcTimeValue};
use utoipa::ToSchema;

use crate::SelectedWeekdays;

/// Timestamp value in seconds which is
/// in inclusive range `[0, (SECONDS_IN_DAY - 1)]`.
///
/// This serializes to i32, so this must not be added to API doc without
/// `#[schema(value_type = i32)]`.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Integer)]
#[serde(try_from = "i32")]
#[serde(into = "i32")]
pub struct DayTimestamp {
    value: i32,
}

impl DayTimestamp {
    pub const MIN: i32 = 0;
    pub const MAX: i32 = (24 * 60 * 60) - 1;

    pub fn new_clamped(value: i32) -> Self {
        Self {
            value: value.clamp(Self::MIN, Self::MAX),
        }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn from_hours(hours: u8) -> Self {
        Self::new_clamped(Into::<i32>::into(hours) * 60 * 60)
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

impl TryFrom<i32> for DayTimestamp {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < Self::MIN || value > Self::MAX {
            Err(format!(
                "DayTimestamp must be in range [{}, {}]",
                Self::MIN,
                Self::MAX
            ))
        } else {
            Ok(Self { value })
        }
    }
}

impl From<DayTimestamp> for i32 {
    fn from(value: DayTimestamp) -> Self {
        value.value
    }
}

impl AsRef<i32> for DayTimestamp {
    fn as_ref(&self) -> &i32 {
        &self.value
    }
}

diesel_i32_wrapper!(DayTimestamp);

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
    #[schema(value_type = i32)]
    pub daily_enabled_time_start_seconds: DayTimestamp,
    #[schema(value_type = i32)]
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

    pub fn field_names_of_true_values(&self) -> String {
        let mut result = String::new();
        if self.moderate_initial_media_content_bot {
            result.push_str("moderate_initial_media_content_bot\n");
        }
        if self.moderate_initial_media_content_human {
            result.push_str("moderate_initial_media_content_human\n");
        }
        if self.moderate_media_content_bot {
            result.push_str("moderate_media_content_bot\n");
        }
        if self.moderate_media_content_human {
            result.push_str("moderate_media_content_human\n");
        }
        if self.moderate_profile_texts_bot {
            result.push_str("moderate_profile_texts_bot\n");
        }
        if self.moderate_profile_texts_human {
            result.push_str("moderate_profile_texts_human\n");
        }
        if self.moderate_profile_names_bot {
            result.push_str("moderate_profile_names_bot\n");
        }
        if self.moderate_profile_names_human {
            result.push_str("moderate_profile_names_human\n");
        }
        if self.process_reports {
            result.push_str("process_reports\n");
        }
        if result.ends_with('\n') {
            result.pop();
        }
        result
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AdminNotificationBitflags {
    bits: i64,
}

impl AdminNotificationBitflags {
    pub const MODERATE_INITIAL_MEDIA_CONTENT_BOT: i64 = 1 << 0;
    pub const MODERATE_INITIAL_MEDIA_CONTENT_HUMAN: i64 = 1 << 1;
    pub const MODERATE_MEDIA_CONTENT_BOT: i64 = 1 << 2;
    pub const MODERATE_MEDIA_CONTENT_HUMAN: i64 = 1 << 3;
    pub const MODERATE_PROFILE_TEXTS_BOT: i64 = 1 << 4;
    pub const MODERATE_PROFILE_TEXTS_HUMAN: i64 = 1 << 5;
    pub const MODERATE_PROFILE_NAMES_BOT: i64 = 1 << 6;
    pub const MODERATE_PROFILE_NAMES_HUMAN: i64 = 1 << 7;
    pub const PROCESS_REPORTS: i64 = 1 << 8;

    pub fn from_bits_truncate(bits: i64) -> Self {
        Self { bits }
    }

    pub fn bits(self) -> i64 {
        self.bits
    }

    pub fn into_admin_notification(self) -> AdminNotification {
        AdminNotification {
            moderate_initial_media_content_bot: self.bits
                & Self::MODERATE_INITIAL_MEDIA_CONTENT_BOT
                != 0,
            moderate_initial_media_content_human: self.bits
                & Self::MODERATE_INITIAL_MEDIA_CONTENT_HUMAN
                != 0,
            moderate_media_content_bot: self.bits & Self::MODERATE_MEDIA_CONTENT_BOT != 0,
            moderate_media_content_human: self.bits & Self::MODERATE_MEDIA_CONTENT_HUMAN != 0,
            moderate_profile_texts_bot: self.bits & Self::MODERATE_PROFILE_TEXTS_BOT != 0,
            moderate_profile_texts_human: self.bits & Self::MODERATE_PROFILE_TEXTS_HUMAN != 0,
            moderate_profile_names_bot: self.bits & Self::MODERATE_PROFILE_NAMES_BOT != 0,
            moderate_profile_names_human: self.bits & Self::MODERATE_PROFILE_NAMES_HUMAN != 0,
            process_reports: self.bits & Self::PROCESS_REPORTS != 0,
        }
    }
}

impl From<AdminNotification> for AdminNotificationBitflags {
    fn from(value: AdminNotification) -> Self {
        let mut bits = 0;

        if value.moderate_initial_media_content_bot {
            bits |= Self::MODERATE_INITIAL_MEDIA_CONTENT_BOT;
        }
        if value.moderate_initial_media_content_human {
            bits |= Self::MODERATE_INITIAL_MEDIA_CONTENT_HUMAN;
        }
        if value.moderate_media_content_bot {
            bits |= Self::MODERATE_MEDIA_CONTENT_BOT;
        }
        if value.moderate_media_content_human {
            bits |= Self::MODERATE_MEDIA_CONTENT_HUMAN;
        }
        if value.moderate_profile_texts_bot {
            bits |= Self::MODERATE_PROFILE_TEXTS_BOT;
        }
        if value.moderate_profile_texts_human {
            bits |= Self::MODERATE_PROFILE_TEXTS_HUMAN;
        }
        if value.moderate_profile_names_bot {
            bits |= Self::MODERATE_PROFILE_NAMES_BOT;
        }
        if value.moderate_profile_names_human {
            bits |= Self::MODERATE_PROFILE_NAMES_HUMAN;
        }
        if value.process_reports {
            bits |= Self::PROCESS_REPORTS;
        }

        Self { bits }
    }
}

impl From<AdminNotificationBitflags> for AdminNotification {
    fn from(value: AdminNotificationBitflags) -> Self {
        value.into_admin_notification()
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

bitflags::bitflags! {
    /// Bot-only moderation notification types
    #[derive(Debug, Clone, Copy, Default)]
    pub struct AdminBotNotificationTypes: u8 {
        const MODERATE_INITIAL_MEDIA_CONTENT_BOT = 1 << 0;
        const MODERATE_MEDIA_CONTENT_BOT = 1 << 1;
        const MODERATE_PROFILE_TEXTS_BOT = 1 << 2;
        const MODERATE_PROFILE_NAMES_BOT = 1 << 3;
        const VERIFY_MEDIA_CONTENT_FACE_BOT = 1 << 4;
        const VERIFY_ACCOUNT_BOT = 1 << 5;
        // TODO(quality): Don't send this when custom report is saved
        const PROCESS_REPORTS = 1 << 6;
    }
}

impl AdminBotNotificationTypes {
    pub fn merge(&self, other: &Self) -> Self {
        *self | *other
    }
}

impl TryFrom<AdminNotificationTypes> for AdminBotNotificationTypes {
    type Error = ();

    fn try_from(value: AdminNotificationTypes) -> Result<Self, Self::Error> {
        match value {
            AdminNotificationTypes::ModerateInitialMediaContentBot => {
                Ok(AdminBotNotificationTypes::MODERATE_INITIAL_MEDIA_CONTENT_BOT)
            }
            AdminNotificationTypes::ModerateMediaContentBot => {
                Ok(AdminBotNotificationTypes::MODERATE_MEDIA_CONTENT_BOT)
            }
            AdminNotificationTypes::ModerateProfileTextsBot => {
                Ok(AdminBotNotificationTypes::MODERATE_PROFILE_TEXTS_BOT)
            }
            AdminNotificationTypes::ModerateProfileNamesBot => {
                Ok(AdminBotNotificationTypes::MODERATE_PROFILE_NAMES_BOT)
            }
            AdminNotificationTypes::ProcessReports => {
                Ok(AdminBotNotificationTypes::PROCESS_REPORTS)
            }
            _ => Err(()),
        }
    }
}
