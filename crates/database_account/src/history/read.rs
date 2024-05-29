use database::ConnectionProvider;

use self::{
    account::HistorySyncReadAccount, account_admin::HistorySyncReadAccountAdmin,
};

pub mod account;
pub mod account_admin;

pub struct HistorySyncReadCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> HistorySyncReadCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_account(self) -> HistorySyncReadAccount<C> {
        HistorySyncReadAccount::new(self.conn)
    }

    pub fn into_account_admin(self) -> HistorySyncReadAccountAdmin<C> {
        HistorySyncReadAccountAdmin::new(self.conn)
    }
}
