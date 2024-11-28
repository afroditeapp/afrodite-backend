use database::DbWriteAccessProviderHistory;

use self::{media::HistoryWriteMedia, media_admin::HistoryWriteMediaAdmin};

pub mod media;
pub mod media_admin;

pub trait GetDbHistoryWriteCommandsMedia {
    fn media_history(&mut self) -> HistoryWriteMedia;
    fn media_admin_history(&mut self) -> HistoryWriteMediaAdmin;
}

impl <I: DbWriteAccessProviderHistory> GetDbHistoryWriteCommandsMedia for I {
    fn media_history(&mut self) -> HistoryWriteMedia {
        HistoryWriteMedia::new(self.handle())
    }
    fn media_admin_history(&mut self) -> HistoryWriteMediaAdmin {
        HistoryWriteMediaAdmin::new(self.handle())
    }
}
