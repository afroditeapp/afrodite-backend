use database::DbWriteAccessProviderHistory;
use profile_admin::HistoryWriteProfileAdmin;

use self::profile::HistoryWriteProfile;

pub mod profile;
pub mod profile_admin;

pub trait GetDbHistoryWriteCommandsProfile {
    fn profile_history(&mut self) -> HistoryWriteProfile;
    fn profile_admin_history(&mut self) -> HistoryWriteProfileAdmin;
}

impl <I: DbWriteAccessProviderHistory> GetDbHistoryWriteCommandsProfile for I {
    fn profile_history(&mut self) -> HistoryWriteProfile {
        HistoryWriteProfile::new(self.handle())
    }
    fn profile_admin_history(&mut self) -> HistoryWriteProfileAdmin {
        HistoryWriteProfileAdmin::new(self.handle())
    }
}
