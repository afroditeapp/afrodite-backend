//! Send events to connected or not connected clients.

use database_chat::current::write::chat::ChatStateChanges;
use model::{
    AccountId, AccountIdInternal, EventToClient, EventToClientInternal, NotificationEvent, PendingNotificationFlags
};
use server_common::{data::IntoDataError, push_notifications::PushNotificationSender};
use tokio::sync::mpsc::{self, error::TrySendError};
use tracing::error;

use crate::{
    cache::DatabaseCache,
    result::{Result, WrappedResultExt},
    DataError,
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

#[derive(Debug, Clone)]
pub enum InternalEventType {
    NormalEvent(EventToClientInternal),
    Notification(NotificationEvent),
}

impl InternalEventType {
    pub fn to_client_event(&self) -> EventToClient {
        match self.clone() {
            InternalEventType::NormalEvent(event) => event.into(),
            InternalEventType::Notification(event) => {
                let event: EventToClientInternal = event.into();
                event.into()
            }
        }
    }
}

#[derive(Debug)]
pub struct EventSender {
    sender: mpsc::Sender<InternalEventType>,
}

pub struct EventReceiver {
    receiver: mpsc::Receiver<InternalEventType>,
}

impl EventReceiver {
    /// Returns None if channel is closed.
    pub async fn recv(&mut self) -> Option<InternalEventType> {
        self.receiver.recv().await
    }
}

pub struct EventManagerWithCacheReference<'a> {
    cache: &'a DatabaseCache,
    push_notification_sender: &'a PushNotificationSender,
}

impl<'a> EventManagerWithCacheReference<'a> {
    pub fn new(
        cache: &'a DatabaseCache,
        push_notification_sender: &'a PushNotificationSender,
    ) -> Self {
        Self {
            cache,
            push_notification_sender,
        }
    }

    async fn access_connection_event_sender<T: Send + 'static>(
        &'a self,
        id: model::AccountId,
        action: impl FnOnce(Option<&EventSender>) -> T + Send,
    ) -> Result<T, DataError> {
        self.cache
            .read_cache(id, move |entry| action(entry.current_connection.as_ref().map(|info| &info.event_sender)))
            .await
            .into_data_error(id)
    }

    /// Send only if the client is connected.
    ///
    /// Event will be skipped if event queue is full.
    pub async fn send_connected_event(
        &'a self,
        account: impl Into<AccountId>,
        event: EventToClientInternal,
    ) -> Result<(), DataError> {
        self.access_connection_event_sender(account.into(), move |sender| {
            if let Some(sender) = sender {
                // Ignore errors
                let _ = sender.sender.try_send(InternalEventType::NormalEvent(event));
            }
        })
        .await
        .change_context(DataError::EventModeAccessFailed)
    }

    /// Send event to connected client or if not connected
    /// send using push notification.
    pub async fn send_notification(
        &'a self,
        account: AccountIdInternal,
        event: NotificationEvent,
    ) -> Result<(), DataError> {
        self.cache
            .write_cache(account, move |entry| {
                entry.pending_notification_flags |= event.into();
                Ok(())
            })
            .await
            .into_data_error(account)?;

        let sent = self
            .access_connection_event_sender(account.into(), move |sender| {
                if let Some(sender) = sender {
                    match sender.sender.try_send(InternalEventType::Notification(event)) {
                        Ok(()) => true,
                        Err(TrySendError::Closed(_) | TrySendError::Full(_)) => false,
                    }
                } else {
                    false
                }
            })
            .await
            .change_context(DataError::EventModeAccessFailed)?;

        if !sent {
            self.push_notification_sender.send(account)
        }

        Ok(())
    }

    pub async fn trigger_push_notification_sending_check_if_needed(
        &'a self,
        account: AccountIdInternal,
    ) {
        let flags_result = self.cache
            .read_cache(account, move |entry| {
                entry.pending_notification_flags
            })
            .await
            .into_data_error(account);

        match flags_result {
            Ok(flags) => if !flags.is_empty() {
                self.push_notification_sender.send(account);
            }
            Err(e) => error!("Failed to read pending notification flags: {:?}", e),
        }
    }

    pub async fn remove_specific_pending_notification_flags_from_cache(
        &'a self,
        account: AccountIdInternal,
        flags: PendingNotificationFlags,
    ) {
        let edit_result = self.cache
            .write_cache(account, move |entry| {
                entry.pending_notification_flags -= flags;
                Ok(())
            })
            .await
            .into_data_error(account);

        match edit_result {
            Ok(()) => (),
            Err(e) => error!("Failed to edit pending notification flags: {:?}", e),
        }
    }

    pub async fn handle_chat_state_changes(&'a self, c: ChatStateChanges) -> Result<(), DataError> {
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
