use chat_admin::HistoryWriteChatAdmin;
use database::DbWriteAccessProviderHistory;

use self::chat::HistoryWriteChat;

pub mod chat;
pub mod chat_admin;

pub trait GetDbHistoryWriteCommandsChat {
    fn chat_history(&mut self) -> HistoryWriteChat<'_>;
    fn chat_admin_history(&mut self) -> HistoryWriteChatAdmin<'_>;
}

impl<I: DbWriteAccessProviderHistory> GetDbHistoryWriteCommandsChat for I {
    fn chat_history(&mut self) -> HistoryWriteChat<'_> {
        HistoryWriteChat::new(self.handle())
    }
    fn chat_admin_history(&mut self) -> HistoryWriteChatAdmin<'_> {
        HistoryWriteChatAdmin::new(self.handle())
    }
}
