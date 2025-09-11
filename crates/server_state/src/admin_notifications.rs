use std::collections::HashMap;

use model::{AccountIdDb, AccountIdInternal, AdminNotification, AdminNotificationTypes, UnixTime};
use simple_backend_utils::time::DurationValue;
use tokio::sync::{
    RwLock,
    mpsc::{self, Receiver, Sender},
};
use tracing::error;

pub enum AdminNotificationEvent {
    SendNotificationIfNeeded(AdminNotificationTypes),
    RefreshStartTimeWaiter,
}

pub struct AdminNotificationEventReceiver(pub Receiver<AdminNotificationEvent>);

#[derive(Default)]
pub struct AccountSpecificState {
    notification: AdminNotification,
    received: bool,
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

    pub async fn get_unreceived_notification(
        &self,
        id: AccountIdInternal,
    ) -> Option<AdminNotification> {
        self.state
            .read()
            .await
            .state
            .get(id.as_db_id())
            .and_then(|v| {
                if v.received {
                    None
                } else {
                    Some(&v.notification)
                }
            })
            .cloned()
    }

    pub fn write(&self) -> AdminNotificationStateWriteAccess {
        AdminNotificationStateWriteAccess { state: &self.state }
    }
}

pub struct AdminNotificationStateWriteAccess<'a> {
    state: &'a RwLock<State>,
}

impl AdminNotificationStateWriteAccess<'_> {
    pub async fn mark_notification_received_and_return_it(
        &self,
        id: AccountIdInternal,
    ) -> AdminNotification {
        if let Some(s) = self.state.write().await.state.get_mut(id.as_db_id()) {
            s.received = true;
            s.notification.clone()
        } else {
            AdminNotification::default()
        }
    }

    /// Returns true, if notification event should be sent to client
    pub async fn send_if_needed(
        &self,
        id: AccountIdInternal,
        notification: AdminNotification,
    ) -> bool {
        if notification == AdminNotification::default() {
            return false;
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
            state.received = false;
            state.sending_time = UnixTime::current_time();
            true
        } else {
            false
        }
    }
}
