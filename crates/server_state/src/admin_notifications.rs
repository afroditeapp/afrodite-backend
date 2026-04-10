use std::collections::HashMap;

use model::{
    AccountIdDb, AccountIdInternal, AdminBotNotificationTypes, AdminNotification,
    AdminNotificationBitflags, AdminNotificationTypes, UnixTime,
};
use simple_backend_utils::time::DurationValue;
use tokio::sync::{
    RwLock,
    mpsc::{self, Receiver, Sender},
};
use tracing::error;

pub enum AdminNotificationEvent {
    SendNotificationIfNeeded(AdminNotificationTypes),
    SendBotNotification(AdminBotNotificationTypes),
    RefreshStartTimeWaiter,
}

pub struct AdminNotificationEventReceiver(pub Receiver<AdminNotificationEvent>);

#[derive(Default)]
pub struct AccountSpecificState {
    notification: AdminNotification,
    sending_time: UnixTime,
}

#[derive(Default)]
struct State {
    state: HashMap<AccountIdDb, AccountSpecificState>,
}

pub struct AdminNotificationManagerData {
    sender: Sender<AdminNotificationEvent>,
    state: RwLock<State>,
}

impl AdminNotificationManagerData {
    pub fn new() -> (Self, AdminNotificationEventReceiver) {
        let (sender, receiver) = mpsc::channel(100);
        let receiver = AdminNotificationEventReceiver(receiver);
        let data = Self {
            sender,
            state: RwLock::default(),
        };
        (data, receiver)
    }

    pub async fn send_notification_if_needed(&self, notification: AdminNotificationTypes) {
        if self
            .sender
            .send(AdminNotificationEvent::SendNotificationIfNeeded(
                notification,
            ))
            .await
            .is_err()
        {
            error!("Send notification if needed event sending failed");
        }
    }

    pub async fn send_bot_notification_if_needed(&self, notification: AdminBotNotificationTypes) {
        if self
            .sender
            .send(AdminNotificationEvent::SendBotNotification(notification))
            .await
            .is_err()
        {
            error!("Send bot notification if needed event sending failed");
        }
    }

    pub async fn refresh_start_time_waiter(&self) {
        if self
            .sender
            .send(AdminNotificationEvent::RefreshStartTimeWaiter)
            .await
            .is_err()
        {
            error!("Refresh start time waiter event sending failed");
        }
    }

    pub fn write(&self) -> AdminNotificationStateWriteAccess<'_> {
        AdminNotificationStateWriteAccess { state: &self.state }
    }
}

pub struct AdminNotificationStateWriteAccess<'a> {
    state: &'a RwLock<State>,
}

impl AdminNotificationStateWriteAccess<'_> {
    /// Returns payload bitflags if notification event should be sent to client.
    pub async fn send_if_needed(
        &self,
        id: AccountIdInternal,
        notification: AdminNotification,
    ) -> Option<AdminNotificationBitflags> {
        if notification == AdminNotification::default() {
            return None;
        }
        let mut write = self.state.write().await;
        let state = write
            .state
            .entry(id.into_db_id())
            .or_insert(AccountSpecificState::default());
        let new_notification = state.notification.merge(&notification);
        if state.notification != new_notification
            || state
                .sending_time
                .duration_value_elapsed(DurationValue::from_days(1))
        {
            state.notification = new_notification;
            state.sending_time = UnixTime::current_time();
            Some(state.notification.clone().into())
        } else {
            None
        }
    }
}
