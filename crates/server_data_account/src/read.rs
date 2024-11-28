use server_data::db_manager::ReadAccessProvider;

use self::{account::ReadCommandsAccount, account_admin::ReadCommandsAccountAdmin};

pub mod account;
pub mod account_admin;

pub trait GetReadCommandsAccount<'a> {
    fn account(self) -> ReadCommandsAccount<'a>;
    fn account_admin(self) -> ReadCommandsAccountAdmin<'a>;
}

impl <'a, T: ReadAccessProvider<'a>> GetReadCommandsAccount<'a> for T {
    fn account(self) -> ReadCommandsAccount<'a> {
        ReadCommandsAccount::new(self.handle())
    }

    fn account_admin(self) -> ReadCommandsAccountAdmin<'a> {
        ReadCommandsAccountAdmin::new(self.handle())
    }
}
