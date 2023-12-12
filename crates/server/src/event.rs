//! Send events to connected or not connected clients.

use std::sync::Arc;

use error_stack::{Result, ResultExt};
use model::{AccountId, EventToClient, EventToClientInternal, NotificationEvent};
use tokio::sync::mpsc;

use crate::data::RouterDatabaseReadHandle;

#[derive(thiserror::Error, Debug)]
pub enum EventError {
    #[error("Event mode access failed")]
    EventModeAccessFailed,
}

pub fn event_channel() -> (EventSender, EventReceiver) {
    let (sender, receiver) = tokio::sync::mpsc::channel(10);
    let sender = EventSender { sender };
    let receiver = EventReceiver { receiver };
    (sender, receiver)
}

#[derive(Debug)]
pub struct EventSender {
    sender: mpsc::Sender<EventToClient>,
}

impl EventSender {
    /// Skips the event if the receiver is full.
    pub fn send(&self, event: EventToClient) {
        let _ = self.sender.try_send(event);
    }
}

pub struct EventReceiver {
    receiver: mpsc::Receiver<EventToClient>,
}

impl EventReceiver {
    /// Returns None if channel is closed.
    pub async fn recv(&mut self) -> Option<EventToClient> {
        self.receiver.recv().await
    }
}

#[derive(Debug)]
pub enum EventMode {
    None,
    Connected(EventSender),
}

pub struct EventManager {
    database: Arc<RouterDatabaseReadHandle>,
}

impl EventManager {
    pub fn new(database: Arc<RouterDatabaseReadHandle>) -> Self {
        Self { database }
    }
    /// Send only if the client is connected.
    pub async fn send_connected_event(
        &self,
        account: impl Into<AccountId>,
        event: EventToClientInternal,
    ) -> Result<(), EventError> {
        self.database
            .read()
            .common()
            .access_event_mode(account.into(), move |mode| {
                if let EventMode::Connected(sender) = mode {
                    let _ = sender.send(event.into());
                }
            })
            .await
            .change_context(EventError::EventModeAccessFailed)
    }

    /// Send event to connected client or if not connected
    /// send using push notification.
    pub async fn send_notification(
        &self,
        account: impl Into<AccountId>,
        event: NotificationEvent,
    ) -> Result<(), EventError> {
        // TODO: Push notification support

        self.database
            .read()
            .common()
            .access_event_mode(account.into(), move |mode| {
                let event: EventToClientInternal = event.into();
                if let EventMode::Connected(sender) = mode {
                    let _ = sender.send(event.into());
                }
            })
            .await
            .change_context(EventError::EventModeAccessFailed)
    }
}
