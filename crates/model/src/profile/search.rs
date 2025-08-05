use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::ToSchema;

use crate::{NotificationIdViewed, NotificationStatus, schema_sqlite_types::Integer};

/// Selected weekdays.
///
/// The integer is a bitflag.
///
/// - const MONDAY = 0x1;
/// - const TUESDAY = 0x2;
/// - const WEDNESDAY = 0x4;
/// - const THURSDAY = 0x8;
/// - const FRIDAY = 0x10;
/// - const SATURDAY = 0x20;
/// - const SUNDAY = 0x40;
///
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Eq,
    Default,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Integer)]
pub struct SelectedWeekdays(i64);

impl SelectedWeekdays {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn as_i64(&self) -> &i64 {
        &self.0
    }

    pub fn all() -> Self {
        Self(WeekdayFlags::all().bits().into())
    }
}

diesel_i64_wrapper!(SelectedWeekdays);

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct WeekdayFlags: i8 {
        const MONDAY = 0x1;
        const TUESDAY = 0x2;
        const WEDNESDAY = 0x4;
        const THURSDAY = 0x8;
        const FRIDAY = 0x10;
        const SATURDAY = 0x20;
        const SUNDAY = 0x40;
    }
}

impl From<SelectedWeekdays> for WeekdayFlags {
    fn from(value: SelectedWeekdays) -> Self {
        value.0.into()
    }
}

impl From<chrono::Weekday> for WeekdayFlags {
    fn from(value: chrono::Weekday) -> Self {
        use chrono::Weekday;
        match value {
            Weekday::Mon => WeekdayFlags::MONDAY,
            Weekday::Tue => WeekdayFlags::TUESDAY,
            Weekday::Wed => WeekdayFlags::WEDNESDAY,
            Weekday::Thu => WeekdayFlags::THURSDAY,
            Weekday::Fri => WeekdayFlags::FRIDAY,
            Weekday::Sat => WeekdayFlags::SATURDAY,
            Weekday::Sun => WeekdayFlags::SUNDAY,
        }
    }
}

impl From<i64> for WeekdayFlags {
    fn from(value: i64) -> Self {
        Self::from_bits_truncate(value as i8)
    }
}

impl From<WeekdayFlags> for i64 {
    fn from(value: WeekdayFlags) -> Self {
        value.bits().into()
    }
}

impl From<WeekdayFlags> for SelectedWeekdays {
    fn from(value: WeekdayFlags) -> Self {
        SelectedWeekdays(value.bits().into())
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, ToSchema)]
pub struct AutomaticProfileSearchCompletedNotification {
    pub profiles_found: NotificationStatus,
}

impl AutomaticProfileSearchCompletedNotification {
    pub fn notifications_viewed(&self) -> bool {
        self.profiles_found.notification_viewed()
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, ToSchema)]
pub struct AutomaticProfileSearchCompletedNotificationViewed {
    pub profiles_found: NotificationIdViewed,
}
