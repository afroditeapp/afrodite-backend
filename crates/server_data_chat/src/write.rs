//! Synchronous write commands combining cache and database operations.

use chat::WriteCommandsChat;
use server_data::db_manager::WriteAccessProvider;

pub mod chat;

pub trait GetWriteCommandsChat<'a> {
    fn chat(self) -> WriteCommandsChat<'a>;
}

impl<'a, I: WriteAccessProvider<'a>> GetWriteCommandsChat<'a> for I {
    fn chat(self) -> WriteCommandsChat<'a> {
        WriteCommandsChat::new(self.handle())
    }
}
