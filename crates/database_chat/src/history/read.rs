use database::define_history_read_commands;

use self::{chat::HistoryReadChat, chat_admin::HistoryReadChatAdmin};

pub mod chat;
pub mod chat_admin;

define_history_read_commands!(HistorySyncReadCommands);

impl<'a> HistorySyncReadCommands<'a> {
    pub fn into_chat(self) -> HistoryReadChat<'a> {
        HistoryReadChat::new(self.cmds)
    }

    pub fn into_chat_admin(self) -> HistoryReadChatAdmin<'a> {
        HistoryReadChatAdmin::new(self.cmds)
    }
}
