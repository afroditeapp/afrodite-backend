use database::DbWriteAccessProvider;
use profile_admin::CurrentWriteProfileAdmin;

use self::profile::CurrentWriteProfile;

pub mod profile;
pub mod profile_admin;

pub trait GetDbWriteCommandsProfile {
    fn profile(&mut self) -> CurrentWriteProfile;
    fn profile_admin(&mut self) -> CurrentWriteProfileAdmin;
}

impl<I: DbWriteAccessProvider> GetDbWriteCommandsProfile for I {
    fn profile(&mut self) -> CurrentWriteProfile {
        CurrentWriteProfile::new(self.handle())
    }
    fn profile_admin(&mut self) -> CurrentWriteProfileAdmin {
        CurrentWriteProfileAdmin::new(self.handle())
    }
}
