use chat::ReadCommandsChat;
use chat_admin::ReadCommandsChatAdmin;
use server_data::{read::ReadCommands};

pub mod chat;
pub mod chat_admin;

pub trait GetReadChatCommands<'a>: Sized {
    fn chat(self) -> ReadCommandsChat<'a>;
    fn chat_admin(self) -> ReadCommandsChatAdmin<'a>;
}

impl <'a> GetReadChatCommands<'a> for ReadCommands<'a> {
    fn chat(self) -> ReadCommandsChat<'a> {
        ReadCommandsChat::new(self)
    }

    fn chat_admin(self) -> ReadCommandsChatAdmin<'a> {
        ReadCommandsChatAdmin::new(self)
    }
}
