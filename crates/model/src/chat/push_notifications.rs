use base64::Engine;
use diesel::{deserialize::Queryable, Selectable};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_wrapper, diesel_string_wrapper};
use utoipa::ToSchema;

use crate::{
    schema_sqlite_types::{Integer, Text},
    NotificationEvent, PublicAccountId,
};

/// Pending notification (or multiple notifications which each have
/// different type) not yet received notifications which push notification
/// requests client to download.
///
/// The integer is a bitflag.
///
/// - const NEW_MESSAGE = 0x1;
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
pub struct PendingNotification(i64);

impl PendingNotification {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn as_i64(&self) -> &i64 {
        &self.0
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
        value.0.into()
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
        PendingNotification(
            value.bits()
        )
    }
}

/// Firebase Cloud Messaging device token.
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    ToSchema,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Text)]
pub struct FcmDeviceToken {
    token: String,
}

impl FcmDeviceToken {
    pub fn into_string(self) -> String {
        self.token
    }

    pub fn new(token: String) -> Self {
        Self { token }
    }

    pub fn as_str(&self) -> &str {
        &self.token
    }
}

diesel_string_wrapper!(FcmDeviceToken);


#[derive(Debug, Selectable, Queryable)]
#[diesel(table_name = crate::schema::chat_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct PendingNotificationTokenRaw {
    pub pending_notification_token: Option<PendingNotificationToken>,
}

/// PendingNotificationToken is used as a token for pending notification
/// API access.
///
/// The token is 256 bit random value which is Base64 encoded.
/// The token lenght in characters is 44.
///
/// OWASP recommends at least 128 bit session IDs.
/// https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = Text)]
pub struct PendingNotificationToken {
    token: String,
}

impl PendingNotificationToken {
    pub fn generate_new() -> Self {
        // Generate 256 bit token
        let mut token = Vec::new();
        for _ in 1..=2 {
            token.extend(uuid::Uuid::new_v4().to_bytes_le())
        }
        Self {
            token: base64::engine::general_purpose::STANDARD.encode(token)
        }
    }

    pub fn new(token: String) -> Self {
        Self {
            token
        }
    }

    pub fn into_string(self) -> String {
        self.token
    }

    pub fn as_str(&self) -> &str {
        &self.token
    }
}

diesel_string_wrapper!(PendingNotificationToken);

/// Pending notification with notification data.
#[derive(
    Debug,
    Clone,
    Default,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
)]
pub struct PendingNotificationWithData {
    pub value: PendingNotification,
    /// Data for NEW_MESSAGE notification.
    ///
    /// List of public account IDs which have sent a new message.
    pub new_message_received_from: Option<Vec<PublicAccountId>>,
}
