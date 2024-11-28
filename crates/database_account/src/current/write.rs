use account_admin::CurrentSyncWriteAccountAdmin;
use database::{ConnectionProvider, DieselConnection};

use self::account::CurrentSyncWriteAccount;

pub mod account;
pub mod account_admin;

pub struct CurrentSyncWriteCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> CurrentSyncWriteCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn read(&mut self) -> database::DbReadMode<'_> {
        database::DbReadMode(self.conn.conn())
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
    pub fn account(&mut self) -> CurrentSyncWriteAccount<&mut DieselConnection> {
        CurrentSyncWriteAccount::new(self.write())
    }

    pub fn account_admin(&mut self) -> CurrentSyncWriteAccountAdmin<&mut DieselConnection> {
        CurrentSyncWriteAccountAdmin::new(self.write())
    }

    pub fn common(
        &mut self,
    ) -> database::current::write::common::CurrentSyncWriteCommon<&mut DieselConnection> {
        database::current::write::common::CurrentSyncWriteCommon::new(self.write())
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
