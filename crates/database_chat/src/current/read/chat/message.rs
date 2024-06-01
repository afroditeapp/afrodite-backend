use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountId, AccountIdInternal, PendingMessage, PendingMessageId, PendingMessageInternal, PublicAccountId,
};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadChatMessage, CurrentSyncReadChatMessage);

impl<C: ConnectionProvider> CurrentSyncReadChatMessage<C> {
    pub fn all_pending_messages(
        &mut self,
        id_message_receiver: AccountIdInternal,
    ) -> Result<Vec<PendingMessage>, DieselDatabaseError> {
        use crate::schema::{account_id, pending_messages::dsl::*};

        let value: Vec<(AccountId, PendingMessageInternal)> = pending_messages
            .inner_join(
                account_id::table.on(account_id_sender.assume_not_null().eq(account_id::id)),
            )
            .filter(account_id_receiver.eq(id_message_receiver.as_db_id()))
            .select((account_id::uuid, PendingMessageInternal::as_select()))
            .load(self.conn())
            .into_db_error(())?;

        let messages = value
            .into_iter()
            .map(|(sender_uuid, msg)| PendingMessage {
                id: PendingMessageId {
                    account_id_sender: sender_uuid,
                    message_number: msg.message_number,
                },
                unix_time: msg.unix_time,
                message: msg.message_text,
            })
            .collect();

        Ok(messages)
    }

    pub fn all_pending_message_sender_public_ids(
        &mut self,
        id_message_receiver: AccountIdInternal,
    ) -> Result<Vec<PublicAccountId>, DieselDatabaseError> {
        use crate::schema::{shared_state, pending_messages::dsl::*};

        let public_ids: Vec<PublicAccountId> = pending_messages
            .inner_join(
                shared_state::table.on(account_id_sender.assume_not_null().eq(shared_state::account_id)),
            )
            .filter(account_id_receiver.eq(id_message_receiver.as_db_id()))
            .select(shared_state::public_uuid)
            .load(self.conn())
            .into_db_error(())?;

        Ok(public_ids)
    }
}
