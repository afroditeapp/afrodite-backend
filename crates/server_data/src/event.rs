//! Send events to connected or not connected clients.

use database_chat::current::write::chat::ChatStateChanges;
use model::{
    AccountId, AccountIdInternal, ContentProcessingStateChanged, EventToClient,
    EventToClientInternal, NotificationEvent, PendingNotificationFlags,
};
use server_common::{data::IntoDataError, push_notifications::PushNotificationSender};
use tokio::sync::mpsc::{self, error::TrySendError};
use tracing::{error, warn};

use crate::{
    DataError,
    cache::DatabaseCache,
    content_processing::ProcessingState,
    result::{Result, WrappedResultExt},
};

/// If [Self::hidden] is true, push notification was sent or sending was tried.
pub struct NotificationVisibility {
    pub hidden: bool,
}

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
            .read_cache_common(id, move |entry| Ok(action(entry.connection_event_sender())))
            .await
            .into_data_error(id)
    }

    async fn access_connection_event_sender_for_logged_in_clients(
        &'a self,
        action: impl Fn(Option<&EventSender>),
    ) {
        self.cache
            .read_cache_common_for_logged_in_clients(|entry| {
                action(entry.connection_event_sender())
            })
            .await
    }

    /// Send only if the client is connected.
    ///
    /// Event will be skipped if event queue is full.
    pub async fn send_connected_event_to_logged_in_clients(&'a self, event: EventToClientInternal) {
        self.access_connection_event_sender_for_logged_in_clients(move |sender| {
            if let Some(sender) = sender {
                // Ignore errors
                let _ = sender
                    .sender
                    .try_send(InternalEventType::NormalEvent(event.clone()));
            }
        })
        .await
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
                let _ = sender
                    .sender
                    .try_send(InternalEventType::NormalEvent(event));
            }
        })
        .await
        .change_context(DataError::EventSenderAccessFailed)
    }

    /// Send event to connected client or if not connected
    /// send using push notification.
    pub async fn send_notification(
        &'a self,
        account: AccountIdInternal,
        event: NotificationEvent,
    ) -> Result<(), DataError> {
        let push_notification_sending_allowed = self
            .cache
            .read_cache_common(account, |entry| {
                Ok(entry.app_notification_settings.get_setting(event))
            })
            .await
            .into_data_error(account)?;

        self.cache
            .write_cache_common(account, move |entry| {
                if push_notification_sending_allowed {
                    entry.pending_notification_flags |= event.into();
                    entry.pending_notification_sent_flags.remove(event.into());
                }
                Ok(())
            })
            .await
            .into_data_error(account)?;

        let sent = self
            .access_connection_event_sender(account.into(), move |sender| {
                if let Some(sender) = sender {
                    match sender
                        .sender
                        .try_send(InternalEventType::Notification(event))
                    {
                        Ok(()) => true,
                        Err(TrySendError::Closed(_) | TrySendError::Full(_)) => false,
                    }
                } else {
                    false
                }
            })
            .await
            .change_context(DataError::EventSenderAccessFailed)?;

        if !sent && push_notification_sending_allowed {
            self.push_notification_sender.send(account)
        }

        Ok(())
    }

    pub async fn send_low_priority_notification_to_logged_in_clients(
        &'a self,
        event: NotificationEvent,
    ) {
        self.cache
            .write_cache_common_for_logged_in_clients(|account_id, entry| {
                let push_notification_sending_allowed =
                    entry.app_notification_settings.get_setting(event);

                if push_notification_sending_allowed {
                    entry.pending_notification_flags |= event.into();
                    entry.pending_notification_sent_flags.remove(event.into());
                }
                let sent = if let Some(sender) = entry.connection_event_sender() {
                    match sender
                        .sender
                        .try_send(InternalEventType::Notification(event))
                    {
                        Ok(()) => true,
                        Err(TrySendError::Closed(_) | TrySendError::Full(_)) => false,
                    }
                } else {
                    false
                };

                if !sent && push_notification_sending_allowed {
                    self.push_notification_sender.send_low_priority(account_id)
                }
            })
            .await;
    }

    pub async fn trigger_push_notification_sending_check_if_needed(
        &'a self,
        account: AccountIdInternal,
    ) {
        let flags_result = self
            .cache
            .read_cache_common(account, move |entry| Ok(entry.pending_notification_flags))
            .await
            .into_data_error(account);

        match flags_result {
            Ok(flags) => {
                if !flags.is_empty() {
                    self.push_notification_sender.send(account);
                }
            }
            Err(e) => error!("Failed to read pending notification flags: {e:?}"),
        }
    }

    pub fn trigger_push_notification_sending(&'a self, account: AccountIdInternal) {
        self.push_notification_sender.send(account);
    }

    /// Also remove the flags from sent flags
    pub async fn remove_specific_pending_notification_flags_from_cache(
        &'a self,
        account: AccountIdInternal,
        flags: PendingNotificationFlags,
    ) -> NotificationVisibility {
        let edit_result = self
            .cache
            .write_cache_common(account, move |entry| {
                entry.pending_notification_flags -= flags;
                let visibility = NotificationVisibility {
                    hidden: entry.pending_notification_sent_flags.contains(flags),
                };
                entry.pending_notification_sent_flags -= flags;
                Ok(visibility)
            })
            .await
            .into_data_error(account);

        match edit_result {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to edit pending notification flags: {e:?}");
                NotificationVisibility { hidden: false }
            }
        }
    }

    /// Also add the returned flags to sent flags
    pub async fn remove_pending_notification_flags_from_cache(
        &'a self,
        account: AccountIdInternal,
    ) -> PendingNotificationFlags {
        let r = self
            .cache
            .write_cache_common(account, move |entry| {
                let flags = entry.pending_notification_flags;
                entry.pending_notification_flags = PendingNotificationFlags::empty();
                entry.pending_notification_sent_flags |= flags;
                Ok(flags)
            })
            .await
            .into_data_error(account);
        match r {
            Ok(flags) => flags,
            Err(e) => {
                error!("Failed to remove pending notification flags: {e:?}");
                PendingNotificationFlags::empty()
            }
        }
    }

    pub async fn handle_chat_state_changes(
        &'a self,
        c: &ChatStateChanges,
    ) -> Result<(), DataError> {
        if let Some(info) = &c.received_likes_change {
            if info.previous_count.c == 0 && info.current_count.c == 1 {
                self.send_notification(c.id, NotificationEvent::ReceivedLikesChanged)
                    .await?;
            } else {
                self.send_connected_event(c.id, EventToClientInternal::ReceivedLikesChanged)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn send_content_processing_state_change_to_client(&self, state: &ProcessingState) {
        let state_change = ContentProcessingStateChanged {
            id: state.processing_id,
            new_state: state.processing_state.clone(),
        };

        if let Err(e) = self
            .send_connected_event(
                state.content_owner,
                model::EventToClientInternal::ContentProcessingStateChanged(state_change),
            )
            .await
        {
            warn!("Event sending failed {e:?}");
        }
    }
}
