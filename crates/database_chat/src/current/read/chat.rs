use database::{DieselDatabaseError, define_current_read_commands};
use diesel::{SelectableHelper, prelude::*};
use error_stack::Result;
use model_chat::{AccountIdInternal, CHAT_GLOBAL_STATE_ROW_TYPE, ChatGlobalState, ChatStateRaw};

use crate::IntoDatabaseError;

mod interaction;
mod limits;
mod message;
mod notification;
mod public_key;

define_current_read_commands!(CurrentReadChat);

impl<'a> CurrentReadChat<'a> {
    pub fn interaction(self) -> interaction::CurrentReadChatInteraction<'a> {
        interaction::CurrentReadChatInteraction::new(self.cmds)
    }

    pub fn message(self) -> message::CurrentReadChatMessage<'a> {
        message::CurrentReadChatMessage::new(self.cmds)
    }

    pub fn public_key(self) -> public_key::CurrentReadChatPublicKey<'a> {
        public_key::CurrentReadChatPublicKey::new(self.cmds)
    }

    pub fn notification(self) -> notification::CurrentReadChatNotification<'a> {
        notification::CurrentReadChatNotification::new(self.cmds)
    }

    pub fn limits(self) -> limits::CurrentReadChatLimits<'a> {
        limits::CurrentReadChatLimits::new(self.cmds)
    }
}

impl CurrentReadChat<'_> {
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

    pub fn global_state(&mut self) -> Result<ChatGlobalState, DieselDatabaseError> {
        use model::schema::chat_global_state::dsl::*;

        chat_global_state
            .filter(row_type.eq(CHAT_GLOBAL_STATE_ROW_TYPE))
            .select(ChatGlobalState::as_select())
            .first(self.conn())
            .optional()
            .map(|v| v.unwrap_or_default())
            .into_db_error(())
    }
}
