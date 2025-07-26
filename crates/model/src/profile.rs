use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_struct_try_from;
use utoipa::ToSchema;

use crate::{NotificationIdViewed, NotificationStatus, schema_sqlite_types::Integer};

mod attribute;
pub use attribute::*;

mod search;
pub use search::*;

/// Profile age value which is in inclusive range `[18, 99]`.
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
#[diesel(sql_type = Integer)]
#[serde(try_from = "i64")]
#[serde(into = "i64")]
pub struct ProfileAge {
    value: u8,
}

impl ProfileAge {
    pub const MIN_AGE: u8 = 18;
    pub const MAX_AGE: u8 = 99;

    pub fn new_clamped(age: u8) -> Self {
        Self {
            value: age.clamp(Self::MIN_AGE, Self::MAX_AGE),
        }
    }
    pub fn value(&self) -> u8 {
        self.value
    }
}

impl Default for ProfileAge {
    fn default() -> Self {
        Self {
            value: Self::MIN_AGE,
        }
    }
}

impl TryFrom<i64> for ProfileAge {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value < Self::MIN_AGE as i64 || value > Self::MAX_AGE as i64 {
            Err(format!(
                "Profile age must be in range [{}, {}]",
                Self::MIN_AGE,
                Self::MAX_AGE
            ))
        } else {
            Ok(Self { value: value as u8 })
        }
    }
}

impl From<ProfileAge> for i64 {
    fn from(value: ProfileAge) -> Self {
        value.value as i64
    }
}

diesel_i64_struct_try_from!(ProfileAge);

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, ToSchema)]
pub struct ProfileStringModerationCompletedNotification {
    pub name_accepted: NotificationStatus,
    pub name_rejected: NotificationStatus,
    pub text_accepted: NotificationStatus,
    pub text_rejected: NotificationStatus,
}

impl ProfileStringModerationCompletedNotification {
    pub fn notifications_viewed(&self) -> bool {
        self.name_accepted.notification_viewed()
            && self.name_rejected.notification_viewed()
            && self.text_accepted.notification_viewed()
            && self.text_rejected.notification_viewed()
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, ToSchema)]
pub struct ProfileStringModerationCompletedNotificationViewed {
    pub name_accepted: NotificationIdViewed,
    pub name_rejected: NotificationIdViewed,
    pub text_accepted: NotificationIdViewed,
    pub text_rejected: NotificationIdViewed,
}
