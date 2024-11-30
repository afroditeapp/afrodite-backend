use database::{define_current_read_commands, DieselDatabaseError};
use diesel::{prelude::*, SelectableHelper};
use error_stack::Result;
use model_chat::{
    AccountIdInternal, ChatGlobalState, ChatStateRaw, PublicKey, PublicKeyData, PublicKeyId,
    PublicKeyVersion, CHAT_GLOBAL_STATE_ROW_TYPE,
};

use crate::IntoDatabaseError;

mod interaction;
mod message;

define_current_read_commands!(CurrentReadChat);

impl<'a> CurrentReadChat<'a> {
    pub fn interaction(self) -> interaction::CurrentReadChatInteraction<'a> {
        interaction::CurrentReadChatInteraction::new(self.cmds)
    }

    pub fn message(self) -> message::CurrentReadChatMessage<'a> {
        message::CurrentReadChatMessage::new(self.cmds)
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

    pub fn public_key(
        &mut self,
        account_id_value: AccountIdInternal,
        version: PublicKeyVersion,
    ) -> Result<Option<PublicKey>, DieselDatabaseError> {
        use crate::schema::public_key::dsl::*;

        let query_result: Option<(Option<PublicKeyId>, Option<PublicKeyData>)> = public_key
            .filter(account_id.eq(account_id_value.as_db_id()))
            .filter(public_key_version.eq(version))
            .select((public_key_id, public_key_data))
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        if let Some((Some(id), Some(data))) = query_result {
            Ok(Some(PublicKey { id, version, data }))
        } else {
            Ok(None)
        }
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
