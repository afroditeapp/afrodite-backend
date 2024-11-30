use database::DbReadAccessProvider;

use self::{profile::CurrentReadProfile, profile_admin::CurrentReadProfileAdmin};

pub mod profile;
pub mod profile_admin;

pub trait GetDbReadCommandsProfile {
    fn profile(&mut self) -> CurrentReadProfile<'_>;
    fn profile_admin(&mut self) -> CurrentReadProfileAdmin<'_>;
}

impl<I: DbReadAccessProvider> GetDbReadCommandsProfile for I {
    fn profile(&mut self) -> CurrentReadProfile<'_> {
        CurrentReadProfile::new(self.handle())
    }
    fn profile_admin(&mut self) -> CurrentReadProfileAdmin<'_> {
        CurrentReadProfileAdmin::new(self.handle())
    }
}
