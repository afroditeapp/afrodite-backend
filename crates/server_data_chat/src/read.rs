use chat::ReadCommandsChat;
use server_data::db_manager::ReadAccessProvider;

pub mod chat;

pub trait GetReadChatCommands<'a> {
    fn chat(self) -> ReadCommandsChat<'a>;
}

impl<'a, I: ReadAccessProvider<'a>> GetReadChatCommands<'a> for I {
    fn chat(self) -> ReadCommandsChat<'a> {
        ReadCommandsChat::new(self.handle())
    }
}
