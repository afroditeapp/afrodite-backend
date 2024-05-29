use database::{ConnectionProvider, DieselConnection};

use self::{
    chat::CurrentSyncReadChat, chat_admin::CurrentSyncReadChatAdmin,
};
pub mod chat;
pub mod chat_admin;

pub struct CurrentSyncReadCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> CurrentSyncReadCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_chat(self) -> CurrentSyncReadChat<C> {
        CurrentSyncReadChat::new(self.conn)
    }

    pub fn into_chat_admin(self) -> CurrentSyncReadChatAdmin<C> {
        CurrentSyncReadChatAdmin::new(self.conn)
    }

    pub fn conn(&mut self) -> &mut C {
        &mut self.conn
    }
}

impl CurrentSyncReadCommands<&mut DieselConnection> {
    pub fn chat(&mut self) -> CurrentSyncReadChat<&mut DieselConnection> {
        CurrentSyncReadChat::new(self.conn())
    }

    pub fn chat_admin(&mut self) -> CurrentSyncReadChatAdmin<&mut DieselConnection> {
        CurrentSyncReadChatAdmin::new(self.conn())
    }
}
