//! Synchronous write commands combining cache and database operations.

use chat::WriteCommandsChat;
use server_data::db_manager::WriteAccessProvider;

pub mod chat;

pub trait GetWriteCommandsChat {
    fn chat(&self) -> WriteCommandsChat<'_>;
}

impl<I: WriteAccessProvider> GetWriteCommandsChat for I {
    fn chat(&self) -> WriteCommandsChat<'_> {
        WriteCommandsChat::new(self.handle())
    }
}
