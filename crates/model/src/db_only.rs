use serde::Serialize;

use crate::{
    AccountId, ConversationId, PendingNotificationFlags, PushNotificationDbState,
    PushNotificationDeviceToken,
};

#[derive(Debug)]
pub struct PushNotificationStateInfo {
    pub push_notification_device_token: Option<PushNotificationDeviceToken>,
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
    /// Notification channel ID string for Android client.
    channel: Option<&'static str>,
    /// Notification receiver AccountId for preventing
    /// client to use this notification if client is signed in to
    /// a different account.
    a: String,
}

impl PushNotification {
    pub fn new(account: AccountId, notification: PushNotificationId, title: String) -> Self {
        Self {
            title: Some(title),
            body: None,
            id: (notification as i64).to_string(),
            channel: notification.to_channel_id(),
            a: account.to_string(),
        }
    }

    pub fn remove_notification(account: AccountId, notification: PushNotificationId) -> Self {
        Self {
            title: None,
            body: None,
            id: (notification as i64).to_string(),
            channel: notification.to_channel_id(),
            a: account.to_string(),
        }
    }

    pub fn new_message(account: AccountId, conversation: ConversationId, title: String) -> Self {
        Self {
            title: Some(title),
            body: None,
            id: ((PushNotificationId::FirstNewMessageNotificationId as i64) + conversation.id)
                .to_string(),
            channel: Some("messages"),
            a: account.to_string(),
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

    /// Account ID
    pub fn a(&self) -> &str {
        &self.a
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
