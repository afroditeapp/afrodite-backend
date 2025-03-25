use account_admin::HistoryWriteAccountAdmin;
use database::DbWriteAccessProviderHistory;

use self::account::HistoryWriteAccount;

pub mod account;
pub mod account_admin;

pub trait GetDbHistoryWriteCommandsAccount {
    fn account_history(&mut self) -> HistoryWriteAccount<'_>;
    fn account_admin_history(&mut self) -> HistoryWriteAccountAdmin<'_>;
}

impl<I: DbWriteAccessProviderHistory> GetDbHistoryWriteCommandsAccount for I {
    fn account_history(&mut self) -> HistoryWriteAccount<'_> {
        HistoryWriteAccount::new(self.handle())
    }
    fn account_admin_history(&mut self) -> HistoryWriteAccountAdmin<'_> {
        HistoryWriteAccountAdmin::new(self.handle())
    }
}
