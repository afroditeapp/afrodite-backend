use chat::CurrentWriteChat;
use database::DbWriteAccessProvider;

pub mod chat;

pub trait GetDbWriteCommandsChat {
    fn chat(&mut self) -> CurrentWriteChat<'_>;
}

impl<I: DbWriteAccessProvider> GetDbWriteCommandsChat for I {
    fn chat(&mut self) -> CurrentWriteChat<'_> {
        CurrentWriteChat::new(self.handle())
    }
}
