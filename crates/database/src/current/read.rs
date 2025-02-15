use common::CurrentReadCommon;
use common_admin::CurrentReadCommonAdmin;

use crate::DbReadAccessProvider;

pub mod common;
pub mod common_admin;

pub trait GetDbReadCommandsCommon {
    fn common(&mut self) -> CurrentReadCommon<'_>;
    fn common_admin(&mut self) -> CurrentReadCommonAdmin<'_>;
}

impl<I: DbReadAccessProvider> GetDbReadCommandsCommon for I {
    fn common(&mut self) -> CurrentReadCommon<'_> {
        CurrentReadCommon::new(self.handle())
    }

    fn common_admin(&mut self) -> CurrentReadCommonAdmin<'_> {
        CurrentReadCommonAdmin::new(self.handle())
    }
}
