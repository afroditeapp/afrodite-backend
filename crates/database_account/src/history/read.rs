use account::HistoryReadAccount;
use account_admin::HistoryReadAccountAdmin;
use database::define_history_read_commands;

pub mod account;
pub mod account_admin;

define_history_read_commands!(HistorySyncReadCommands);

impl<'a> HistorySyncReadCommands<'a> {
    pub fn into_account(self) -> HistoryReadAccount<'a> {
        HistoryReadAccount::new(self.cmds)
    }

    pub fn into_account_admin(self) -> HistoryReadAccountAdmin<'a> {
        HistoryReadAccountAdmin::new(self.cmds)
    }
}
