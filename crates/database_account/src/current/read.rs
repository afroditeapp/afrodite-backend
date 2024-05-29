use database::{ConnectionProvider, DieselConnection};

use self::{
    account::CurrentSyncReadAccount, account_admin::CurrentSyncReadAccountAdmin,
};

pub mod account;
pub mod account_admin;

pub struct CurrentSyncReadCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> CurrentSyncReadCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_account(self) -> CurrentSyncReadAccount<C> {
        CurrentSyncReadAccount::new(self.conn)
    }

    pub fn into_account_admin(self) -> CurrentSyncReadAccountAdmin<C> {
        CurrentSyncReadAccountAdmin::new(self.conn)
    }

    pub fn conn(&mut self) -> &mut C {
        &mut self.conn
    }
}

impl CurrentSyncReadCommands<&mut DieselConnection> {
    pub fn account(&mut self) -> CurrentSyncReadAccount<&mut DieselConnection> {
        CurrentSyncReadAccount::new(self.conn())
    }

    pub fn account_admin(&mut self) -> CurrentSyncReadAccountAdmin<&mut DieselConnection> {
        CurrentSyncReadAccountAdmin::new(self.conn())
    }
}
