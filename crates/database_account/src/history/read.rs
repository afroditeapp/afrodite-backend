use account::HistoryReadAccount;
use account_admin::HistoryReadAccountAdmin;
use database::DbReadAccessProviderHistory;

pub mod account;
pub mod account_admin;

pub trait GetDbHistoryReadCommandsAccount {
    fn account_history(&mut self) -> HistoryReadAccount<'_>;
    fn account_admin_history(&mut self) -> HistoryReadAccountAdmin<'_>;
}

impl<I: DbReadAccessProviderHistory> GetDbHistoryReadCommandsAccount for I {
    fn account_history(&mut self) -> HistoryReadAccount<'_> {
        HistoryReadAccount::new(self.handle())
    }
    fn account_admin_history(&mut self) -> HistoryReadAccountAdmin<'_> {
        HistoryReadAccountAdmin::new(self.handle())
    }
}
