use account_admin::CurrentReadAccountAdmin;
use database::DbReadAccessProvider;
use profile::CurrentReadProfileUtils;

use self::account::CurrentReadAccount;

pub mod account;
pub mod account_admin;
pub mod profile;

pub trait GetDbReadCommandsAccount {
    fn account(&mut self) -> CurrentReadAccount<'_>;
    fn account_admin(&mut self) -> CurrentReadAccountAdmin<'_>;
    fn account_profile_utils(&mut self) -> CurrentReadProfileUtils<'_>;
}

impl<I: DbReadAccessProvider> GetDbReadCommandsAccount for I {
    fn account(&mut self) -> CurrentReadAccount<'_> {
        CurrentReadAccount::new(self.handle())
    }

    fn account_admin(&mut self) -> CurrentReadAccountAdmin<'_> {
        CurrentReadAccountAdmin::new(self.handle())
    }

    fn account_profile_utils(&mut self) -> CurrentReadProfileUtils<'_> {
        CurrentReadProfileUtils::new(self.handle())
    }
}
