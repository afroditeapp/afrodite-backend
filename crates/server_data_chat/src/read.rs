use chat::ReadCommandsChat;
use chat_admin::ReadCommandsChatAdmin;
use server_data::db_manager::{InternalReading, ReadAccessProvider};

pub mod chat;
pub mod chat_admin;

pub trait GetReadChatCommands<'a> {
    fn chat(self) -> ReadCommandsChat<'a>;
    fn chat_admin(self) -> ReadCommandsChatAdmin<'a>;
}

impl <'a, I: ReadAccessProvider<'a>> GetReadChatCommands<'a> for I {
    fn chat(self) -> ReadCommandsChat<'a> {
        ReadCommandsChat::new(self.handle())
    }
    fn chat_admin(self) -> ReadCommandsChatAdmin<'a> {
        ReadCommandsChatAdmin::new(self.handle())
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
