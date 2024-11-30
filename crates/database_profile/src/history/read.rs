use database::DbReadAccessProviderHistory;
use profile_admin::HistoryReadProfileAdmin;

pub mod profile;
pub mod profile_admin;

pub trait GetDbReadCommandsProfileHistory {
    fn profile_admin(&mut self) -> HistoryReadProfileAdmin<'_>;
}

impl<I: DbReadAccessProviderHistory> GetDbReadCommandsProfileHistory for I {
    fn profile_admin(&mut self) -> HistoryReadProfileAdmin<'_> {
        HistoryReadProfileAdmin::new(self.handle())
    }
}
