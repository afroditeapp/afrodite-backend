use chat::CurrentWriteChat;
use chat_admin::CurrentWriteChatAdmin;
use database::DbWriteAccessProvider;

pub mod chat;
pub mod chat_admin;

pub trait GetDbWriteCommandsChat {
    fn chat(&mut self) -> CurrentWriteChat;
    fn chat_admin(&mut self) -> CurrentWriteChatAdmin;
}

impl <I: DbWriteAccessProvider> GetDbWriteCommandsChat for I {
    fn chat(&mut self) -> CurrentWriteChat {
        CurrentWriteChat::new(self.handle())
    }
    fn chat_admin(&mut self) -> CurrentWriteChatAdmin {
        CurrentWriteChatAdmin::new(self.handle())
    }
}
