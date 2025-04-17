use base64::Engine;
use diesel::{deserialize::Queryable, Selectable};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_wrapper, diesel_string_wrapper};
use utils::random_bytes::random_128_bits;
use utoipa::ToSchema;

use crate::{
    schema_sqlite_types::{Integer, Text}, AccountId, AutomaticProfileSearchCompletedNotification, MediaContentModerationCompletedNotification, NewReceivedLikesCountResult, NotificationEvent, ProfileTextModerationCompletedNotification, UnreadNewsCountResult
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
/// - const PROFILE_TEXT_MODERATION_COMPLETED = 0x10;
/// - const AUTOMATIC_PROFILE_SEARCH_COMPLETED = 0x20;
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
        const PROFILE_TEXT_MODERATION_COMPLETED = 0x10;
        const AUTOMATIC_PROFILE_SEARCH_COMPLETED = 0x20;
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
            NotificationEvent::MediaContentModerationCompleted =>
                Self::MEDIA_CONTENT_MODERATION_COMPLETED,
            NotificationEvent::NewsChanged => Self::NEWS_CHANGED,
            NotificationEvent::ProfileTextModerationCompleted => Self::PROFILE_TEXT_MODERATION_COMPLETED,
            NotificationEvent::AutomaticProfileSearchCompleted => Self::AUTOMATIC_PROFILE_SEARCH_COMPLETED,
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
    pub value: PendingNotification,
    /// Data for NEW_MESSAGE notification.
    ///
    /// List of account IDs which have sent a new message.
    pub new_message_received_from: Option<Vec<AccountId>>,
    /// Data for RECEIVED_LIKES_CHANGED notification.
    pub received_likes_changed: Option<NewReceivedLikesCountResult>,
    /// Data for MEDIA_CONTENT_MODERATION_COMPLETED notification.
    pub media_content_moderation_completed: Option<MediaContentModerationCompletedNotification>,
    /// Data for NEWS_CHANGED notification.
    pub news_changed: Option<UnreadNewsCountResult>,
    /// Data for PROFILE_TEXT_MODERATION_COMPLETED notification.
    pub profile_text_moderation_completed: Option<ProfileTextModerationCompletedNotification>,
    /// Data for AUTOMATIC_PROFILE_SEARCH_COMPLETED notification.
    pub automatic_profile_search_completed: Option<AutomaticProfileSearchCompletedNotification>,
}

#[derive(Debug, Clone, Default, Queryable, Selectable)]
#[diesel(table_name = crate::schema::common_state)]
#[diesel(check_for_backend(crate::Db))]
#[diesel(treat_none_as_null = true)]
pub struct PushNotificationDbState {
    pub pending_notification: PendingNotification,
    pub fcm_notification_sent: bool,
    pub fcm_device_token: Option<FcmDeviceToken>,
}
