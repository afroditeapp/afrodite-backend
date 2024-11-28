//! Synchronous write commands combining cache and database operations.

use chat::WriteCommandsChat;
use chat_admin::WriteCommandsChatAdmin;
use server_data::db_manager::WriteAccessProvider;

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
