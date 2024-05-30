use server_data::read::{ReadCommandsProvider};
use self::{
    account::ReadCommandsAccount, account_admin::ReadCommandsAccountAdmin
};

pub mod account;
pub mod account_admin;

pub trait GetReadCommandsAccount<C: ReadCommandsProvider> {
    fn account(self) -> ReadCommandsAccount<C>;
    fn account_admin(self) -> ReadCommandsAccountAdmin<C>;
}

impl <C: ReadCommandsProvider> GetReadCommandsAccount<C> for C {
    fn account(self) -> ReadCommandsAccount<C> {
        ReadCommandsAccount::new(self)
    }

    fn account_admin(self) -> ReadCommandsAccountAdmin<C> {
        ReadCommandsAccountAdmin::new(self)
    }
}
