//! Synchronous write commands combining cache and database operations.

use chat::WriteCommandsChat;
use chat_admin::WriteCommandsChatAdmin;
use server_data::db_manager::WriteAccessProvider;

pub mod chat;
pub mod chat_admin;

pub trait GetWriteCommandsChat {
    fn chat(&self) -> WriteCommandsChat<'_>;
    fn chat_admin(&self) -> WriteCommandsChatAdmin<'_>;
}

impl<I: WriteAccessProvider> GetWriteCommandsChat for I {
    fn chat(&self) -> WriteCommandsChat<'_> {
        WriteCommandsChat::new(self.handle())
    }
    fn chat_admin(&self) -> WriteCommandsChatAdmin<'_> {
        WriteCommandsChatAdmin::new(self.handle())
    }
}
