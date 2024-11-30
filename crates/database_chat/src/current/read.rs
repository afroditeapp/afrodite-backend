use database::DbReadAccessProvider;

use self::{chat::CurrentReadChat, chat_admin::CurrentReadChatAdmin};
pub mod chat;
pub mod chat_admin;

pub trait GetDbReadCommandsChat {
    fn chat(&mut self) -> CurrentReadChat<'_>;
    fn chat_admin(&mut self) -> CurrentReadChatAdmin<'_>;
}

impl<I: DbReadAccessProvider> GetDbReadCommandsChat for I {
    fn chat(&mut self) -> CurrentReadChat {
        CurrentReadChat::new(self.handle())
    }

    fn chat_admin(&mut self) -> CurrentReadChatAdmin {
        CurrentReadChatAdmin::new(self.handle())
    }
}
