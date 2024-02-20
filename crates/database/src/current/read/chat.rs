use diesel::SelectableHelper;
use model::{AccountIdInternal, ChatStateRaw};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use error_stack::Result;
use diesel::prelude::*;
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

    pub fn chat_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<ChatStateRaw, DieselDatabaseError> {
        use crate::schema::chat_state::dsl::*;

        chat_state
            .filter(account_id.eq(id.as_db_id()))
            .select(ChatStateRaw::as_select())
            .first(self.conn())
            .into_db_error(())
    }
}
