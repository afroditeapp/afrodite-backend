use account_admin_history::ReadCommandsAccountAdminHistory;
use profile::ReadCommandsProfileUtils;
use server_data::db_manager::ReadAccessProvider;

use self::{account::ReadCommandsAccount, account_admin::ReadCommandsAccountAdmin};

pub mod account;
pub mod account_admin;
pub mod account_admin_history;
pub mod profile;

pub trait GetReadCommandsAccount<'a> {
    fn account(self) -> ReadCommandsAccount<'a>;
    fn account_admin(self) -> ReadCommandsAccountAdmin<'a>;
    fn account_admin_history(self) -> ReadCommandsAccountAdminHistory<'a>;
    fn account_profile_utils(self) -> ReadCommandsProfileUtils<'a>;
}

impl<'a, T: ReadAccessProvider<'a>> GetReadCommandsAccount<'a> for T {
    fn account(self) -> ReadCommandsAccount<'a> {
        ReadCommandsAccount::new(self.handle())
    }

    fn account_admin(self) -> ReadCommandsAccountAdmin<'a> {
        ReadCommandsAccountAdmin::new(self.handle())
    }

    fn account_admin_history(self) -> ReadCommandsAccountAdminHistory<'a> {
        ReadCommandsAccountAdminHistory::new(self.handle())
    }

    fn account_profile_utils(self) -> ReadCommandsProfileUtils<'a> {
        ReadCommandsProfileUtils::new(self.handle())
    }
}
