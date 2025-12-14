use database::DbWriteAccessProviderHistory;

use self::{media::HistoryWriteMedia, media_admin::HistoryWriteMediaAdmin};

pub mod media;
pub mod media_admin;

pub trait GetDbHistoryWriteCommandsMedia {
    fn media_history(&mut self) -> HistoryWriteMedia<'_>;
    fn media_admin_history(&mut self) -> HistoryWriteMediaAdmin<'_>;
}

impl<I: DbWriteAccessProviderHistory> GetDbHistoryWriteCommandsMedia for I {
    fn media_history(&mut self) -> HistoryWriteMedia<'_> {
        HistoryWriteMedia::new(self.handle())
    }
    fn media_admin_history(&mut self) -> HistoryWriteMediaAdmin<'_> {
        HistoryWriteMediaAdmin::new(self.handle())
    }
}
