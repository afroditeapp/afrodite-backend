use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::{prelude::*, SelectableHelper};
use error_stack::Result;
use model::{AccountIdInternal, ChatStateRaw, PublicKey, PublicKeyData, PublicKeyId, PublicKeyIdAndVersion, PublicKeyVersion};

use crate::IntoDatabaseError;

mod interaction;
mod message;

define_current_read_commands!(CurrentReadChat, CurrentSyncReadChat);

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

    pub fn public_key(
        &mut self,
        account_id_value: AccountIdInternal,
        version: PublicKeyVersion,
    ) -> Result<Option<PublicKey>, DieselDatabaseError> {
        use crate::schema::public_key::dsl::*;

        let query_result: Option<(Option<PublicKeyId>, Option<PublicKeyData>)> = public_key
            .filter(account_id.eq(account_id_value.as_db_id()))
            .filter(public_key_version.eq(version))
            .select((
                public_key_id,
                public_key_data
            ))
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        if let Some((Some(id), Some(data))) =
            query_result {
                Ok(Some(
                    PublicKey {
                        id,
                        version,
                        data
                    }
                ))
            } else {
                Ok(None)
            }
    }

    pub fn get_latest_public_keys_info(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<Vec<PublicKeyIdAndVersion>, DieselDatabaseError> {
        use crate::schema::public_key::dsl::*;

        let query_result: Vec<(PublicKeyId, PublicKeyVersion)> = public_key
            .filter(account_id.eq(account_id_value.as_db_id()))
            .filter(public_key_id.is_not_null())
            .select((
                public_key_id.assume_not_null(),
                public_key_version
            ))
            .load(self.conn())
            .into_db_error(())?;

        let info_list = query_result
            .into_iter()
            .map(|(id, version)| {
                PublicKeyIdAndVersion {
                    id,
                    version,
                }
            })
            .collect::<Vec<_>>();

        Ok(info_list)
    }
}
