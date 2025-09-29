use serde::{Serialize, ser::Error};

use crate::{
    AccountId, ConversationId, FcmDeviceToken, PendingNotificationFlags,
    PendingNotificationWithData, PushNotificationDbState,
};

#[derive(Debug)]
pub struct PushNotificationStateInfo {
    pub fcm_device_token: Option<FcmDeviceToken>,
}

pub enum PushNotificationStateInfoWithFlags {
    EmptyFlags,
    WithFlags {
        info: PushNotificationStateInfo,
        flags: PendingNotificationFlags,
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
    #[serde(serialize_with = "serialize_payload_as_string")]
    payload: NotificationPayload,
    /// Notification channel ID string for Android client.
    channel: Option<&'static str>,
}

fn serialize_payload_as_string<S>(
    payload: &NotificationPayload,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let json_string = serde_json::to_string(payload).map_err(S::Error::custom)?;
    serializer.serialize_str(&json_string)
}

impl PushNotification {
    pub fn new(
        account: AccountId,
        notification: PushNotificationId,
        title: String,
        data: PendingNotificationWithData,
    ) -> Self {
        Self {
            title: Some(title),
            body: None,
            id: (notification as i64).to_string(),
            payload: NotificationPayload {
                a: account.to_string(),
                data,
            },
            channel: notification.to_channel_id(),
        }
    }

    pub fn remove_notification(
        account: AccountId,
        notification: PushNotificationId,
        data: PendingNotificationWithData,
    ) -> Self {
        Self {
            title: None,
            body: None,
            id: (notification as i64).to_string(),
            payload: NotificationPayload {
                a: account.to_string(),
                data,
            },
            channel: notification.to_channel_id(),
        }
    }

    pub fn new_message(
        account: AccountId,
        conversation: ConversationId,
        title: String,
        data: PendingNotificationWithData,
    ) -> Self {
        Self {
            title: Some(title),
            body: None,
            id: ((PushNotificationId::FirstNewMessageNotificationId as i64) + conversation.id)
                .to_string(),
            payload: NotificationPayload {
                a: account.to_string(),
                data,
            },
            channel: Some("messages"),
        }
    }

    pub fn is_visible(&self) -> bool {
        self.title.is_some()
    }

    pub fn id(&self) -> String {
        self.id.clone()
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

#[derive(Serialize)]
pub struct NotificationPayload {
    /// Notification receiver AccountId for preventing
    /// client to use this payload if client is signed in to
    /// a different account.
    a: String,
    /// Notification related state which client should store
    /// to prevent the same notification showing again
    /// when WebSocket connects.
    data: PendingNotificationWithData,
}
