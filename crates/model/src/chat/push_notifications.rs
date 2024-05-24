
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_wrapper, diesel_string_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::schema_sqlite_types::Text;

use crate::NotificationEvent;
use crate::{schema::shared_state, schema_sqlite_types::Integer, AccessToken, AccountIdDb, AccountIdInternal, AccountSyncVersion, RefreshToken, SharedStateRaw};

/// Pending notification (or multiple notifications which each have
/// different type) not yet received notifications which push notification
/// requests client to download.
///
/// The integer is a bitflag.
///
/// - const NEW_MESSAGE = 0x1;
///
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Eq, Default, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = Integer)]
pub struct PendingNotification {
    pub value: i64,
}

impl PendingNotification {
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn as_i64(&self) -> &i64 {
       &self.value
    }
}

diesel_i64_wrapper!(PendingNotification);

bitflags::bitflags! {
    /// Profile mood filter
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PendingNotificationFlags: i64 {
        const NEW_MESSAGE = 0x1;
    }
}

impl From<PendingNotification> for PendingNotificationFlags {
    fn from(value: PendingNotification) -> Self {
        value.value.into()
    }
}

impl From<NotificationEvent> for PendingNotificationFlags {
    fn from(value: NotificationEvent) -> Self {
        match value {
            NotificationEvent::NewMessageReceived => Self::NEW_MESSAGE,
        }
    }
}

impl From<i64> for PendingNotificationFlags {
    fn from(value: i64) -> Self {
        Self::from_bits_truncate(value)
    }
}

impl From<PendingNotificationFlags> for i64 {
    fn from(value: PendingNotificationFlags) -> Self {
        value.bits()
    }
}

impl From<PendingNotificationFlags> for PendingNotification {
    fn from(value: PendingNotificationFlags) -> Self {
        PendingNotification { value: value.bits() }
    }
}

/// Firebase Cloud Messaging device token.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = Text)]
pub struct FcmDeviceToken {
    value: String,
}

impl FcmDeviceToken {
    pub fn into_string(self) -> String {
        self.value
    }

    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

diesel_string_wrapper!(FcmDeviceToken);
