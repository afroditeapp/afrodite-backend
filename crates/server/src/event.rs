//! Send events to connected or not connected clients.

use database::current::write::chat::ChatStateChanges;
use model::{AccountId, AccountIdInternal, EventToClient, EventToClientInternal, FcmDeviceToken, NotificationEvent};
use tokio::sync::mpsc;

use crate::{
    data::{cache::DatabaseCache, DataError, IntoDataError}, push_notifications::PushNotificationSender, result::{Result, WrappedResultExt}
};

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

pub struct EventManagerWithCacheReference<'a> {
    cache: &'a DatabaseCache,
    push_notification_sender: &'a PushNotificationSender,
}

impl <'a> EventManagerWithCacheReference<'a> {
    pub fn new(
        cache: &'a DatabaseCache,
        push_notification_sender: &'a PushNotificationSender,
    ) -> Self {
        Self {
            cache,
            push_notification_sender,
        }
    }

    async fn access_event_mode<T>(
        &self,
        id: AccountId,
        action: impl FnOnce(&EventMode) -> T,
    ) -> Result<T, DataError> {
        self.cache
            .read_cache(
                id,
                move |entry| action(
                    &entry.current_event_connection,
                )
            )
            .await
            .into_data_error(id)
    }

    /// Send only if the client is connected.
    ///
    /// Event will be skipped if event queue is full.
    pub async fn send_connected_event(
        &self,
        account: impl Into<AccountId>,
        event: EventToClientInternal,
    ) -> Result<(), DataError> {
        self.access_event_mode(account.into(), move |mode| {
                if let EventMode::Connected(sender) = mode {
                    sender.send(event.into())
                }
            })
            .await
            .change_context(DataError::EventModeAccessFailed)
    }

    /// Send event to connected client or if not connected
    /// send using push notification.
    pub async fn send_notification(
        &self,
        account: AccountIdInternal,
        event: NotificationEvent,
    ) -> Result<(), DataError> {
        let sent =
            self.access_event_mode(account.into(), move |mode| {
                let event: EventToClientInternal = event.into();
                if let EventMode::Connected(sender) = mode {
                    sender.send(event.into());
                    true
                } else {
                    false
                }
            })
            .await
            .change_context(DataError::EventModeAccessFailed)?;

        if !sent {
            self.push_notification_sender
                .send(account, event)
        }

        Ok(())
    }

    pub async fn handle_chat_state_changes(
        &self,
        c: ChatStateChanges,
    ) -> Result<(), DataError> {
        if c.received_blocks_sync_version.is_some() {
            self.send_connected_event(c.id, EventToClientInternal::ReceivedBlocksChanged)
                .await?;
        }
        if c.received_likes_sync_version.is_some() {
            self.send_connected_event(c.id, EventToClientInternal::ReceivedLikesChanged)
                .await?;
        }
        if c.sent_likes_sync_version.is_some() {
            self.send_connected_event(c.id, EventToClientInternal::SentLikesChanged)
                .await?;
        }
        if c.sent_blocks_sync_version.is_some() {
            self.send_connected_event(c.id, EventToClientInternal::SentBlocksChanged)
                .await?;
        }
        if c.matches_sync_version.is_some() {
            self.send_connected_event(c.id, EventToClientInternal::MatchesChanged)
                .await?;
        }

        Ok(())
    }
}
