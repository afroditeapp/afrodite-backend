use base64::{Engine, prelude::BASE64_STANDARD};
use diesel::{
    Selectable,
    deserialize::{FromSqlRow, Queryable},
    expression::AsExpression,
    sql_types::SmallInt,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_db_i16_is_i8_struct, diesel_i64_wrapper, diesel_string_wrapper};
use utils::random_bytes::random_128_bits;
use utoipa::ToSchema;

use crate::{
    ConversationId, NotificationEvent,
    schema_sqlite_types::{Integer, Text},
    sync_version_wrappers,
};

/// Push notification type. Backend uses this internally for
/// tracking which notifications should be send and which notifications
/// should be displayed when WebSocket connects.
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
pub struct PushNotificationFlagsDb(i64);

impl PushNotificationFlagsDb {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn as_i64(&self) -> &i64 {
        &self.0
    }
}

diesel_i64_wrapper!(PushNotificationFlagsDb);

bitflags::bitflags! {
    /// If you add anything here, remember to remove the flag from the
    /// cache so that websocket code do not send push notification when it
    /// is not needed.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PushNotificationFlags: i64 {
        const NEW_MESSAGE = 0x1;
        const RECEIVED_LIKES_CHANGED = 0x2;
        const MEDIA_CONTENT_MODERATION_COMPLETED = 0x4;
        const NEWS_CHANGED = 0x8;
        const PROFILE_STRING_MODERATION_COMPLETED = 0x10;
        const AUTOMATIC_PROFILE_SEARCH_COMPLETED = 0x20;
        const ADMIN_NOTIFICATION = 0x40;
    }
}

impl From<PushNotificationFlagsDb> for PushNotificationFlags {
    fn from(value: PushNotificationFlagsDb) -> Self {
        value.0.into()
    }
}

impl From<NotificationEvent> for PushNotificationFlags {
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

impl From<i64> for PushNotificationFlags {
    fn from(value: i64) -> Self {
        Self::from_bits_truncate(value)
    }
}

impl From<PushNotificationFlags> for i64 {
    fn from(value: PushNotificationFlags) -> Self {
        value.bits()
    }
}

impl From<PushNotificationFlags> for PushNotificationFlagsDb {
    fn from(value: PushNotificationFlags) -> Self {
        PushNotificationFlagsDb(value.bits())
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
pub struct PushNotificationDeviceToken {
    token: String,
}

impl PushNotificationDeviceToken {
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

diesel_string_wrapper!(PushNotificationDeviceToken);

/// 128 bit random value which is Base64 encoded.
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
pub struct PushNotificationEncryptionKey {
    key: String,
}

impl PushNotificationEncryptionKey {
    pub fn generate_new() -> Self {
        Self {
            key: base64::engine::general_purpose::STANDARD.encode(random_128_bits()),
        }
    }

    pub fn new(key: String) -> Self {
        Self { key }
    }

    pub fn into_string(self) -> String {
        self.key
    }

    pub fn as_str(&self) -> &str {
        &self.key
    }
}

diesel_string_wrapper!(PushNotificationEncryptionKey);

#[derive(Debug, Clone, Default, Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::push_notification)]
#[diesel(check_for_backend(crate::Db))]
#[diesel(treat_none_as_null = true)]
pub struct PushNotificationDbState {
    pub pending_flags: PushNotificationFlagsDb,
    pub sent_flags: PushNotificationFlagsDb,
    pub encryption_key: Option<PushNotificationEncryptionKey>,
    pub device_token: Option<PushNotificationDeviceToken>,
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
#[diesel(sql_type = SmallInt)]
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

impl From<NotificationId> for i16 {
    fn from(value: NotificationId) -> Self {
        value.id.into()
    }
}

impl TryFrom<i16> for NotificationId {
    type Error = std::num::TryFromIntError;
    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Ok(Self {
            id: TryInto::try_into(value)?,
        })
    }
}

diesel_db_i16_is_i8_struct!(NotificationId);

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
#[diesel(sql_type = SmallInt)]
pub struct NotificationIdViewed {
    pub id: i8,
}

impl From<NotificationIdViewed> for i16 {
    fn from(value: NotificationIdViewed) -> Self {
        value.id.into()
    }
}

impl TryFrom<i16> for NotificationIdViewed {
    type Error = std::num::TryFromIntError;
    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Ok(Self {
            id: TryInto::try_into(value)?,
        })
    }
}

diesel_db_i16_is_i8_struct!(NotificationIdViewed);

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

/// Base64 encoded VAPID public key
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VapidPublicKey {
    key: String,
}

impl VapidPublicKey {
    pub fn new(public_key: &[u8]) -> Self {
        Self {
            key: BASE64_STANDARD.encode(public_key),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GetPushNotificationInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_token: Option<PushNotificationDeviceToken>,
    /// Base64 encoded VAPID public key if web push notifications
    /// are enabled and current login session if from web client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vapid_public_key: Option<VapidPublicKey>,
    pub sync_version: PushNotificationInfoSyncVersion,
}

sync_version_wrappers!(PushNotificationInfoSyncVersion,);

#[derive(Debug)]
pub struct PushNotificationStateInfo {
    pub push_notification_device_token: Option<PushNotificationDeviceToken>,
}

pub enum PushNotificationStateInfoWithFlags {
    EmptyFlags,
    WithFlags {
        info: PushNotificationStateInfo,
        flags: PushNotificationFlags,
    },
}

pub struct PushNotificationSendingInfo {
    pub db_state: PushNotificationDbState,
    pub notifications: Vec<PushNotification>,
}

#[derive(Serialize)]
pub struct PushNotification {
    /// If None, notification should be hidden
    title: Option<String>,
    body: Option<String>,
    /// Notification ID number which client can
    /// use to hide the notification or run notification
    /// specific navigation action.
    id: String,
    /// Notification channel ID string for Android client.
    channel: Option<&'static str>,
}

impl PushNotification {
    pub fn new(notification: PushNotificationId, title: String) -> Self {
        Self {
            title: Some(title),
            body: None,
            id: (notification as i64).to_string(),
            channel: notification.to_channel_id(),
        }
    }

    pub fn new_with_body(notification: PushNotificationId, title: String, body: String) -> Self {
        Self {
            title: Some(title),
            body: Some(body),
            id: (notification as i64).to_string(),
            channel: notification.to_channel_id(),
        }
    }

    pub fn remove_notification(notification: PushNotificationId) -> Self {
        Self {
            title: None,
            body: None,
            id: (notification as i64).to_string(),
            channel: notification.to_channel_id(),
        }
    }

    pub fn new_message(conversation: ConversationId, title: String) -> Self {
        Self {
            title: Some(title),
            body: None,
            id: ((PushNotificationId::FirstNewMessageNotificationId as i64) + conversation.id)
                .to_string(),
            channel: Some("messages"),
        }
    }

    pub fn is_visible(&self) -> bool {
        self.title.is_some()
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    pub fn channel(&self) -> Option<&str> {
        self.channel
    }
}

/// Notification IDs from client
#[derive(Clone, Copy)]
pub enum PushNotificationId {
    // Backend does not use this
    // NotificationDecryptingFailed = 0,

    // Common
    AdminNotification = 10,

    // Account
    NewsItemAvailable = 20,

    // Profile
    ProfileNameModerationAccepted = 30,
    ProfileNameModerationRejected = 31,
    ProfileTextModerationAccepted = 32,
    ProfileTextModerationRejected = 33,
    AutomaticProfileSearchCompleted = 34,

    // Media
    MediaContentModerationAccepted = 40,
    MediaContentModerationRejected = 41,
    MediaContentModerationDeleted = 42,

    // Chat
    LikeReceived = 50,
    // Backend does not use this
    // GenericMessageReceived = 51,
    FirstNewMessageNotificationId = 1000,
}

impl PushNotificationId {
    /// Convert to Android notification channel ID
    fn to_channel_id(self) -> Option<&'static str> {
        match self {
            Self::AdminNotification | Self::NewsItemAvailable => Some("news_item_available"),
            Self::ProfileNameModerationAccepted
            | Self::ProfileNameModerationRejected
            | Self::ProfileTextModerationAccepted
            | Self::ProfileTextModerationRejected => Some("profile_string_moderation_completed"),
            Self::AutomaticProfileSearchCompleted => Some("automatic_profile_search"),
            Self::MediaContentModerationAccepted
            | Self::MediaContentModerationRejected
            | Self::MediaContentModerationDeleted => Some("media_content_moderation_completed"),
            Self::LikeReceived => Some("likes"),
            Self::FirstNewMessageNotificationId => None,
        }
    }
}
