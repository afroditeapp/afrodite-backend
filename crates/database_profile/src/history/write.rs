use database::DbWriteAccessProviderHistory;
use profile_admin::HistoryWriteProfileAdmin;

use self::profile::HistoryWriteProfile;

pub mod profile;
pub mod profile_admin;

pub trait GetDbHistoryWriteCommandsProfile {
    fn profile_history(&mut self) -> HistoryWriteProfile<'_>;
    fn profile_admin_history(&mut self) -> HistoryWriteProfileAdmin<'_>;
}

impl<I: DbWriteAccessProviderHistory> GetDbHistoryWriteCommandsProfile for I {
    fn profile_history(&mut self) -> HistoryWriteProfile<'_> {
        HistoryWriteProfile::new(self.handle())
    }
    fn profile_admin_history(&mut self) -> HistoryWriteProfileAdmin<'_> {
        HistoryWriteProfileAdmin::new(self.handle())
    }
}
