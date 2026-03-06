use database_chat::current::write::chat::ChatStateChanges;
use model::{EventToClientInternal, NotificationEvent};
use server_data::{DataError, event::EventManagerWithCacheReference, result::Result};

pub trait EventManagerChatMethods {
    async fn handle_chat_state_changes(&self, c: &ChatStateChanges) -> Result<(), DataError>;
}

impl EventManagerChatMethods for EventManagerWithCacheReference<'_> {
    async fn handle_chat_state_changes(&self, c: &ChatStateChanges) -> Result<(), DataError> {
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
}
