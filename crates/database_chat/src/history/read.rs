use database::ConnectionProvider;

use self::{
    chat::HistorySyncReadChat, chat_admin::HistorySyncReadChatAdmin,
};

pub mod chat;
pub mod chat_admin;

pub struct HistorySyncReadCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> HistorySyncReadCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_chat(self) -> HistorySyncReadChat<C> {
        HistorySyncReadChat::new(self.conn)
    }

    pub fn into_chat_admin(self) -> HistorySyncReadChatAdmin<C> {
        HistorySyncReadChatAdmin::new(self.conn)
    }
}
