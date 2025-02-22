use chat::CurrentWriteChat;
use database::DbWriteAccessProvider;

pub mod chat;

pub trait GetDbWriteCommandsChat {
    fn chat(&mut self) -> CurrentWriteChat;
}

impl<I: DbWriteAccessProvider> GetDbWriteCommandsChat for I {
    fn chat(&mut self) -> CurrentWriteChat {
        CurrentWriteChat::new(self.handle())
    }
}
