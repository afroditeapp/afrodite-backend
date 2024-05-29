use chat::CurrentSyncWriteChat;
use database::{TransactionError};
use database::{
    ConnectionProvider, DieselConnection, DieselDatabaseError,
};

pub mod chat;
pub mod chat_admin;

pub struct CurrentSyncWriteCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> CurrentSyncWriteCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn read(&mut self) -> crate::current::read::CurrentSyncReadCommands<&mut DieselConnection> {
        crate::current::read::CurrentSyncReadCommands::new(self.conn.conn())
    }

    pub fn write(&mut self) -> &mut C {
        &mut self.conn
    }

    pub fn conn(&mut self) -> &mut DieselConnection {
        self.conn.conn()
    }
}

/// Write commands for current database. All commands must be run in
/// a database transaction.
impl CurrentSyncWriteCommands<&mut DieselConnection> {
    pub fn chat(&mut self) -> CurrentSyncWriteChat<&mut DieselConnection> {
        CurrentSyncWriteChat::new(self.write())
    }

    pub fn transaction<
        F: FnOnce(
            &mut DieselConnection,
        ) -> std::result::Result<T, TransactionError>,
        T,
    >(
        self,
        transaction_actions: F,
    ) -> error_stack::Result<T, DieselDatabaseError> {
        use diesel::prelude::*;
        self.conn.transaction(transaction_actions)
            .map_err(|e| e.into_report())
    }
}

pub struct TransactionConnection<'a> {
    conn: &'a mut DieselConnection,
}

impl<'a> TransactionConnection<'a> {
    pub fn new(conn: &'a mut DieselConnection) -> Self {
        Self { conn }
    }

    pub fn into_conn(self) -> &'a mut DieselConnection {
        self.conn
    }

    pub fn into_cmds(self) -> CurrentSyncWriteCommands<&'a mut DieselConnection> {
        CurrentSyncWriteCommands::new(self.conn)
    }
}

impl ConnectionProvider for &mut TransactionConnection<'_> {
    fn conn(&mut self) -> &mut DieselConnection {
        self.conn
    }
}
