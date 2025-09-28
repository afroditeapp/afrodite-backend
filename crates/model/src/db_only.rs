use serde::Serialize;

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
    ///
    /// Notification library in the client will add this to
    /// the [Self::payload] object.
    id: i64,
    payload: NotificationPayload,
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
            id: notification as i64,
            payload: NotificationPayload {
                a: account.to_string(),
                data,
            },
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
            id: notification as i64,
            payload: NotificationPayload {
                a: account.to_string(),
                data,
            },
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
            id: (PushNotificationId::FirstNewMessageNotificationId as i64) + conversation.id,
            payload: NotificationPayload {
                a: account.to_string(),
                data,
            },
        }
    }

    pub fn is_visible(&self) -> bool {
        self.title.is_some()
    }

    pub fn id(&self) -> i64 {
        self.id
    }
}

pub enum PushNotificationId {
    NotificationDecryptingFailed = 0,

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
    GenericMessageReceived = 51,
    FirstNewMessageNotificationId = 1000,
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
