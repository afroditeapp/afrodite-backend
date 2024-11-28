//! Synchronous write commands combining cache and database operations.

use chat::WriteCommandsChat;
use chat_admin::WriteCommandsChatAdmin;
use server_data::db_manager::{InternalWriting, WriteAccessProvider};

pub mod chat;
pub mod chat_admin;

pub trait GetWriteCommandsChat<'a> {
    fn chat(self) -> WriteCommandsChat<'a>;
    fn chat_admin(self) -> WriteCommandsChatAdmin<'a>;
}

impl <'a, I: WriteAccessProvider<'a>> GetWriteCommandsChat<'a> for I {
    fn chat(self) -> WriteCommandsChat<'a> {
        WriteCommandsChat::new(self.handle())
    }
    fn chat_admin(self) -> WriteCommandsChatAdmin<'a> {
        WriteCommandsChatAdmin::new(self.handle())
    }
}

pub trait DbTransactionChat {
    async fn db_transaction<
        T: FnOnce(
                database_chat::current::write::CurrentSyncWriteCommands<
                    &mut server_data::DieselConnection,
                >,
            ) -> error_stack::Result<R, server_data::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, server_data::DieselDatabaseError>;
}

impl <I: InternalWriting> DbTransactionChat for I {
    async fn db_transaction<
        T: FnOnce(
                database_chat::current::write::CurrentSyncWriteCommands<
                    &mut server_data::DieselConnection,
                >,
            ) -> error_stack::Result<R, server_data::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, server_data::DieselDatabaseError> {
        self.db_transaction_raw(|conn| cmd(database_chat::current::write::CurrentSyncWriteCommands::new(conn))).await
    }
}
