use chat_admin::HistoryWriteChatAdmin;
use database::DbWriteAccessProviderHistory;

use self::chat::HistoryWriteChat;

pub mod chat;
pub mod chat_admin;

pub trait GetDbHistoryWriteCommandsChat {
    fn chat_history(&mut self) -> HistoryWriteChat;
    fn chat_admin_history(&mut self) -> HistoryWriteChatAdmin;
}

impl <I: DbWriteAccessProviderHistory> GetDbHistoryWriteCommandsChat for I {
    fn chat_history(&mut self) -> HistoryWriteChat {
        HistoryWriteChat::new(self.handle())
    }
    fn chat_admin_history(&mut self) -> HistoryWriteChatAdmin {
        HistoryWriteChatAdmin::new(self.handle())
    }
}
