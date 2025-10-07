use base64::Engine;
use diesel::{
    Selectable,
    deserialize::{FromSqlRow, Queryable},
    expression::AsExpression,
    sql_types::BigInt,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{
    UnixTime, diesel_i64_struct_try_from, diesel_i64_wrapper, diesel_string_wrapper,
};
use utils::random_bytes::random_128_bits;
use utoipa::ToSchema;

use crate::{
    AdminNotification, AutomaticProfileSearchCompletedNotification, NewMessageNotificationList,
    NewReceivedLikesCountResult, NotificationEvent, UnreadNewsCountResult,
    schema_sqlite_types::{Integer, Text},
    sync_version_wrappers,
};

/// Pending notification (or multiple notifications which each have
/// different type) not yet received notifications which push notification
/// requests client to download.
///
/// The integer is a bitflag.
///
/// - const NEW_MESSAGE = 0x1;
/// - const RECEIVED_LIKES_CHANGED = 0x2;
/// - const MEDIA_CONTENT_MODERATION_COMPLETED = 0x4;
/// - const NEWS_CHANGED = 0x8;
/// - const PROFILE_STRING_MODERATION_COMPLETED = 0x10;
/// - const AUTOMATIC_PROFILE_SEARCH_COMPLETED = 0x20;
/// - const ADMIN_NOTIFICATION = 0x40;
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
    /// If you add anything here, remember to remove the flag from the
    /// cache so that websocket code do not send push notification when it
    /// is not needed.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PendingNotificationFlags: i64 {
        const NEW_MESSAGE = 0x1;
        const RECEIVED_LIKES_CHANGED = 0x2;
        const MEDIA_CONTENT_MODERATION_COMPLETED = 0x4;
        const NEWS_CHANGED = 0x8;
        const PROFILE_STRING_MODERATION_COMPLETED = 0x10;
        const AUTOMATIC_PROFILE_SEARCH_COMPLETED = 0x20;
        const ADMIN_NOTIFICATION = 0x40;
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
            NotificationEvent::ReceivedLikesChanged => Self::RECEIVED_LIKES_CHANGED,
            NotificationEvent::MediaContentModerationCompleted => {
                Self::MEDIA_CONTENT_MODERATION_COMPLETED
            }
            NotificationEvent::NewsChanged => Self::NEWS_CHANGED,
            NotificationEvent::ProfileStringModerationCompleted => {
                Self::PROFILE_STRING_MODERATION_COMPLETED
            }
            NotificationEvent::AutomaticProfileSearchCompleted => {
                Self::AUTOMATIC_PROFILE_SEARCH_COMPLETED
            }
            NotificationEvent::AdminNotification => Self::ADMIN_NOTIFICATION,
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
        PendingNotification(value.bits())
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
#[diesel(table_name = crate::schema::common_state)]
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
#[derive(
    Debug,
    Deserialize,
    Serialize,
    ToSchema,
    Clone,
    Eq,
    Hash,
    PartialEq,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Text)]
pub struct PendingNotificationToken {
    token: String,
}

impl PendingNotificationToken {
    pub fn generate_new() -> Self {
        // Generate 256 bit token
        let mut token = Vec::new();
        for _ in 1..=2 {
            token.extend(random_128_bits())
        }
        Self {
            token: base64::engine::general_purpose::STANDARD.encode(token),
        }
    }

    pub fn new(token: String) -> Self {
        Self { token }
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
#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct PendingNotificationWithData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_message: Option<NewMessageNotificationList>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub received_likes_changed: Option<NewReceivedLikesCountResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_content_accepted: Option<NotificationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_content_rejected: Option<NotificationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_content_deleted: Option<NotificationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub news_changed: Option<UnreadNewsCountResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_name_accepted: Option<NotificationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_name_rejected: Option<NotificationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_text_accepted: Option<NotificationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_text_rejected: Option<NotificationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automatic_profile_search_completed: Option<AutomaticProfileSearchCompletedNotification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_notification: Option<AdminNotification>,
}

#[derive(Debug, Clone, Default, Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::common_state)]
#[diesel(check_for_backend(crate::Db))]
#[diesel(treat_none_as_null = true)]
pub struct PushNotificationDbState {
    pub pending_notification: PendingNotification,
    pub fcm_device_token: Option<FcmDeviceToken>,
    pub fcm_device_token_unix_time: Option<UnixTime>,
}

/// Notification ID for an event. Can be used to prevent showing
/// the same notification again. The ID is i8 number which will wrap.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Deserialize,
    Serialize,
    PartialEq,
    FromSqlRow,
    AsExpression,
    ToSchema,
)]
#[diesel(sql_type = BigInt)]
pub struct NotificationId {
    pub id: i8,
}

impl NotificationId {
    pub fn wrapping_increment(self) -> Self {
        Self {
            id: self.id.wrapping_add(1),
        }
    }
}

impl From<NotificationId> for i64 {
    fn from(value: NotificationId) -> Self {
        value.id.into()
    }
}

impl TryFrom<i64> for NotificationId {
    type Error = std::num::TryFromIntError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self {
            id: TryInto::try_into(value)?,
        })
    }
}

diesel_i64_struct_try_from!(NotificationId);

/// Notification ID which client has handled.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Deserialize,
    Serialize,
    PartialEq,
    FromSqlRow,
    AsExpression,
    ToSchema,
)]
#[diesel(sql_type = BigInt)]
pub struct NotificationIdViewed {
    pub id: i8,
}

impl From<NotificationIdViewed> for i64 {
    fn from(value: NotificationIdViewed) -> Self {
        value.id.into()
    }
}

impl TryFrom<i64> for NotificationIdViewed {
    type Error = std::num::TryFromIntError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self {
            id: TryInto::try_into(value)?,
        })
    }
}

diesel_i64_struct_try_from!(NotificationIdViewed);

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, ToSchema)]
pub struct NotificationStatus {
    pub id: NotificationId,
    pub viewed: NotificationIdViewed,
}

impl NotificationStatus {
    pub fn notification_viewed(&self) -> bool {
        self.id.id == self.viewed.id
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetVapidPublicKey {
    /// Base64 encoded VAPID public key if web push notifications
    /// are enabled.
    pub vapid_public_key: Option<String>,
}

sync_version_wrappers!(PushNotificationInfoSyncVersion,);
