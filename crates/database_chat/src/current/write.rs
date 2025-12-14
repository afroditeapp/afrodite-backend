use chat::CurrentWriteChat;
use chat_admin::CurrentWriteChatAdmin;
use database::DbWriteAccessProvider;

pub mod chat;
pub mod chat_admin;

pub trait GetDbWriteCommandsChat {
    fn chat(&mut self) -> CurrentWriteChat<'_>;
    fn chat_admin(&mut self) -> CurrentWriteChatAdmin<'_>;
}

impl<I: DbWriteAccessProvider> GetDbWriteCommandsChat for I {
    fn chat(&mut self) -> CurrentWriteChat<'_> {
        CurrentWriteChat::new(self.handle())
    }
    fn chat_admin(&mut self) -> CurrentWriteChatAdmin<'_> {
        CurrentWriteChatAdmin::new(self.handle())
    }
}
