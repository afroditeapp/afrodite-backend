use diesel::sql_types::SmallInt;
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i16_wrapper;
use utoipa::ToSchema;

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
#[diesel(sql_type = SmallInt)]
pub struct SelectedWeekdays(i16);

impl SelectedWeekdays {
    pub fn all() -> Self {
        Self(WeekdayFlags::all().bits())
    }
}

impl TryFrom<i16> for SelectedWeekdays {
    type Error = String;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

impl AsRef<i16> for SelectedWeekdays {
    fn as_ref(&self) -> &i16 {
        &self.0
    }
}

diesel_i16_wrapper!(SelectedWeekdays);

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct WeekdayFlags: i16 {
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

impl From<jiff::civil::Weekday> for WeekdayFlags {
    fn from(value: jiff::civil::Weekday) -> Self {
        use jiff::civil::Weekday;
        match value {
            Weekday::Monday => WeekdayFlags::MONDAY,
            Weekday::Tuesday => WeekdayFlags::TUESDAY,
            Weekday::Wednesday => WeekdayFlags::WEDNESDAY,
            Weekday::Thursday => WeekdayFlags::THURSDAY,
            Weekday::Friday => WeekdayFlags::FRIDAY,
            Weekday::Saturday => WeekdayFlags::SATURDAY,
            Weekday::Sunday => WeekdayFlags::SUNDAY,
        }
    }
}

impl From<i16> for WeekdayFlags {
    fn from(value: i16) -> Self {
        Self::from_bits_truncate(value)
    }
}

impl From<WeekdayFlags> for i16 {
    fn from(value: WeekdayFlags) -> Self {
        value.bits()
    }
}

impl From<WeekdayFlags> for SelectedWeekdays {
    fn from(value: WeekdayFlags) -> Self {
        SelectedWeekdays(value.bits())
    }
}
