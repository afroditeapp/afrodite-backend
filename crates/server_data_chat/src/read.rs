use chat::ReadCommandsChat;
use chat_admin::ReadCommandsChatAdmin;
use server_data::db_manager::ReadAccessProvider;

pub mod chat;
pub mod chat_admin;

pub trait GetReadChatCommands<'a> {
    fn chat(self) -> ReadCommandsChat<'a>;
    fn chat_admin(self) -> ReadCommandsChatAdmin<'a>;
}

impl<'a, I: ReadAccessProvider<'a>> GetReadChatCommands<'a> for I {
    fn chat(self) -> ReadCommandsChat<'a> {
        ReadCommandsChat::new(self.handle())
    }
    fn chat_admin(self) -> ReadCommandsChatAdmin<'a> {
        ReadCommandsChatAdmin::new(self.handle())
    }
}
