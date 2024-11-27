//! Synchronous write commands combining cache and database operations.

use chat::WriteCommandsChat;
use chat_admin::WriteCommandsChatAdmin;
use server_data::db_manager::{InternalWriting, WriteAccessProvider};

pub mod chat;
pub mod chat_admin;

pub trait GetWriteCommandsChat: Sized {
    fn chat(self) -> WriteCommandsChat<Self>;
    fn chat_admin(self) -> WriteCommandsChatAdmin<Self>;
}

impl <I: WriteAccessProvider> GetWriteCommandsChat for I {
    fn chat(self) -> WriteCommandsChat<Self> {
        WriteCommandsChat::new(self)
    }
    fn chat_admin(self) -> WriteCommandsChatAdmin<Self> {
        WriteCommandsChatAdmin::new(self)
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
