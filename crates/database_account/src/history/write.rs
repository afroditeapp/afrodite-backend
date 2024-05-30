use database::{ConnectionProvider, DieselConnection, DieselDatabaseError, TransactionError};

use self::account::HistorySyncWriteAccount;

pub mod account;
pub mod account_admin;

pub struct HistorySyncWriteCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> HistorySyncWriteCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_account(self) -> HistorySyncWriteAccount<C> {
        HistorySyncWriteAccount::new(self.conn)
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
    pub fn account(&mut self) -> HistorySyncWriteAccount<&mut DieselConnection> {
        HistorySyncWriteAccount::new(self.write())
    }

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
