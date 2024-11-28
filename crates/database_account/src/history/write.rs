use database::DbWriteAccessProviderHistory;

use self::account::HistoryWriteAccount;

pub mod account;
pub mod account_admin;

pub trait GetDbHistoryWriteCommandsAccount {
    fn account_history(&mut self) -> HistoryWriteAccount<'_>;
}

impl <I: DbWriteAccessProviderHistory> GetDbHistoryWriteCommandsAccount for I {
    fn account_history(&mut self) -> HistoryWriteAccount<'_> {
        HistoryWriteAccount::new(self.handle())
    }
}
