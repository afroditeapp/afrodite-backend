use database::DbReadAccessProvider;

use self::chat::CurrentReadChat;
pub mod chat;

pub trait GetDbReadCommandsChat {
    fn chat(&mut self) -> CurrentReadChat<'_>;
}

impl<I: DbReadAccessProvider> GetDbReadCommandsChat for I {
    fn chat(&mut self) -> CurrentReadChat<'_> {
        CurrentReadChat::new(self.handle())
    }
}
