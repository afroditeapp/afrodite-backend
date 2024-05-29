use server_data::{read::ReadCommands};
use self::{
    account::ReadCommandsAccount, account_admin::ReadCommandsAccountAdmin
};

pub mod account;
pub mod account_admin;

pub trait GetReadCommandsAccount<'a>: Sized {
    fn account(self) -> ReadCommandsAccount<'a>;
    fn account_admin(self) -> ReadCommandsAccountAdmin<'a>;
}

impl <'a> GetReadCommandsAccount<'a> for ReadCommands<'a> {
    fn account(self) -> ReadCommandsAccount<'a> {
        ReadCommandsAccount::new(self)
    }

    fn account_admin(self) -> ReadCommandsAccountAdmin<'a> {
        ReadCommandsAccountAdmin::new(self)
    }
}
