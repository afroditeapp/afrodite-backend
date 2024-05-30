use database::{ConnectionProvider, DieselConnection, DieselDatabaseError, TransactionError};

use self::chat::HistorySyncWriteChat;

pub mod chat;
pub mod chat_admin;

pub struct HistorySyncWriteCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> HistorySyncWriteCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_chat(self) -> HistorySyncWriteChat<C> {
        HistorySyncWriteChat::new(self.conn)
    }

    // pub fn read(&mut self) -> crate::history::read::HistorySyncReadCommands<&mut DieselConnection> {
    //     self.conn.read()
    // }

    pub fn write(&mut self) -> &mut C {
        &mut self.conn
    }

    pub fn conn(&mut self) -> &mut DieselConnection {
        self.conn.conn()
    }
}

impl HistorySyncWriteCommands<&mut DieselConnection> {
    pub fn transaction<
        F: FnOnce(&mut DieselConnection) -> std::result::Result<T, TransactionError> + 'static,
        T,
    >(
        self,
        transaction_actions: F,
    ) -> error_stack::Result<T, DieselDatabaseError> {
        use diesel::prelude::*;
        self.conn
            .transaction(transaction_actions)
            .map_err(|e| e.into_report())
    }
}
