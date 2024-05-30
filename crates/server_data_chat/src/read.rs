use chat::ReadCommandsChat;
use chat_admin::ReadCommandsChatAdmin;
use server_data::read::ReadCommandsProvider;

pub mod chat;
pub mod chat_admin;

pub trait GetReadChatCommands<C: ReadCommandsProvider> {
    fn chat(self) -> ReadCommandsChat<C>;
    fn chat_admin(self) -> ReadCommandsChatAdmin<C>;
}

impl<C: ReadCommandsProvider> GetReadChatCommands<C> for C {
    fn chat(self) -> ReadCommandsChat<C> {
        ReadCommandsChat::new(self)
    }

    fn chat_admin(self) -> ReadCommandsChatAdmin<C> {
        ReadCommandsChatAdmin::new(self)
    }
}
