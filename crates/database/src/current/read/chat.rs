use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountId, AccountIdInternal, AccountInteractionInternal, AccountInteractionState,
    PendingMessage, PendingMessageId, PendingMessageInternal,
};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};
use tokio_stream::StreamExt;

use crate::IntoDatabaseError;

mod interaction;
mod message;

define_read_commands!(CurrentReadChat, CurrentSyncReadChat);

impl<C: ConnectionProvider> CurrentSyncReadChat<C> {
    pub fn interaction(self) -> interaction::CurrentSyncReadChatInteraction<C> {
        interaction::CurrentSyncReadChatInteraction::new(self.cmds)
    }

    pub fn message(self) -> message::CurrentSyncReadChatMessage<C> {
        message::CurrentSyncReadChatMessage::new(self.cmds)
    }
}
