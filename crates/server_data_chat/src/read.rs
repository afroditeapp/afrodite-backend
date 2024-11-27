use chat::ReadCommandsChat;
use chat_admin::ReadCommandsChatAdmin;
use server_data::db_manager::{InternalReading, ReadAccessProvider};

pub mod chat;
pub mod chat_admin;

pub trait GetReadChatCommands: Sized {
    fn chat(self) -> ReadCommandsChat<Self>;
    fn chat_admin(self) -> ReadCommandsChatAdmin<Self>;
}

impl <I: ReadAccessProvider> GetReadChatCommands for I {
    fn chat(self) -> ReadCommandsChat<Self> {
        ReadCommandsChat::new(self)
    }
    fn chat_admin(self) -> ReadCommandsChatAdmin<Self> {
        ReadCommandsChatAdmin::new(self)
    }
}

pub trait DbReadChat {
    async fn db_read<
        T: FnOnce(
                database_chat::current::read::CurrentSyncReadCommands<
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

impl <I: InternalReading> DbReadChat for I {
    async fn db_read<
        T: FnOnce(
                database_chat::current::read::CurrentSyncReadCommands<
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
        self.db_read_raw(|conn| {
            cmd(database_chat::current::read::CurrentSyncReadCommands::new(
                conn,
            ))
        })
        .await
    }
}
