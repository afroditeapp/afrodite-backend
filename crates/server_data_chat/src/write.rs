//! Synchronous write commands combining cache and database operations.

use chat::WriteCommandsChat;
use chat_admin::WriteCommandsChatAdmin;
use server_data::write::WriteCommands;

pub mod chat;
pub mod chat_admin;

pub trait GetWriteCommandsChat<'a>: Sized {
    fn chat(self) -> WriteCommandsChat<'a>;
    fn chat_admin(self) -> WriteCommandsChatAdmin<'a>;
}

impl <'a> GetWriteCommandsChat<'a> for WriteCommands<'a> {
    fn chat(self) -> WriteCommandsChat<'a> {
        WriteCommandsChat::new(self)
    }

    fn chat_admin(self) -> WriteCommandsChatAdmin<'a> {
        WriteCommandsChatAdmin::new(self)
    }
}
