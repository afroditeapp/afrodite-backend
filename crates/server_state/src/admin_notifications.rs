
use std::collections::HashMap;

use model::{AccountIdDb, AccountIdInternal, AdminNotificationSubscriptions, AdminNotificationTypes};
use tokio::sync::{mpsc::{self, Receiver, Sender}, RwLock};

use tracing::error;

pub enum AdminNotificationEvent {
    ResetState(AccountIdInternal),
    SendNotificationIfNeeded(AdminNotificationTypes),
}

pub struct AdminNotificationEventReceiver(pub Receiver<AdminNotificationEvent>);

#[derive(Default)]
struct State {
    sent_status: HashMap<AccountIdDb, AdminNotificationSubscriptions>,
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

    pub async fn reset_state(&self, id: AccountIdInternal) {
        if self.sender.send(AdminNotificationEvent::ResetState(id)).await.is_err() {
            error!("Reset state event sending failed");
        }
    }

    pub async fn send_notification_if_needed(&self, notification: AdminNotificationTypes) {
        if self.sender.send(AdminNotificationEvent::SendNotificationIfNeeded(notification)).await.is_err() {
            error!("Send notification if needed event sending failed");
        }
    }

    pub async fn get_notification_state(&self, id: AccountIdInternal) -> Option<AdminNotificationSubscriptions> {
        self.state.read().await.sent_status.get(id.as_db_id()).cloned()
    }

    pub async fn reset_notification_state(&self, id: AccountIdInternal) {
        self.state.write().await.sent_status.remove(id.as_db_id());
    }

    pub async fn set_notification_state(&self, id: AccountIdInternal, state: AdminNotificationSubscriptions) {
        self.state.write().await.sent_status.insert(id.into_db_id(), state);
    }
}
