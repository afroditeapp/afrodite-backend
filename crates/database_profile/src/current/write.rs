use database::DbWriteAccessProvider;
use profile_admin::CurrentWriteProfileAdmin;

use self::profile::CurrentWriteProfile;

pub mod profile;
pub mod profile_admin;

pub trait GetDbWriteCommandsProfile {
    fn profile(&mut self) -> CurrentWriteProfile<'_>;
    fn profile_admin(&mut self) -> CurrentWriteProfileAdmin<'_>;
}

impl<I: DbWriteAccessProvider> GetDbWriteCommandsProfile for I {
    fn profile(&mut self) -> CurrentWriteProfile<'_> {
        CurrentWriteProfile::new(self.handle())
    }
    fn profile_admin(&mut self) -> CurrentWriteProfileAdmin<'_> {
        CurrentWriteProfileAdmin::new(self.handle())
    }
}
