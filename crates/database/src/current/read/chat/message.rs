
use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountId, AccountIdInternal, AccountInteractionInternal, AccountInteractionState,
    PendingMessage, PendingMessageId, PendingMessageInternal,
};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};
use tokio_stream::StreamExt;

use crate::IntoDatabaseError;

define_read_commands!(CurrentReadChatMessage, CurrentSyncReadChatMessage);

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
            .into_db_error(DieselDatabaseError::Execute, ())?;

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
}
