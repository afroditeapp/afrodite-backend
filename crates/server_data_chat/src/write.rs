//! Synchronous write commands combining cache and database operations.

use chat::WriteCommandsChat;
use chat_admin::WriteCommandsChatAdmin;
use server_data::write::WriteCommandsProvider;

pub mod chat;
pub mod chat_admin;

pub trait GetWriteCommandsChat<C: WriteCommandsProvider> {
    fn chat(self) -> WriteCommandsChat<C>;
    fn chat_admin(self) -> WriteCommandsChatAdmin<C>;
}

impl<C: WriteCommandsProvider> GetWriteCommandsChat<C> for C {
    fn chat(self) -> WriteCommandsChat<C> {
        WriteCommandsChat::new(self)
    }

    fn chat_admin(self) -> WriteCommandsChatAdmin<C> {
        WriteCommandsChatAdmin::new(self)
    }
}
