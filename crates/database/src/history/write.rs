use common::HistoryWriteCommon;

use crate::DbWriteAccessProviderHistory;

pub mod common;

pub trait GetDbHistoryWriteCommandsCommon {
    fn common_history(&mut self) -> HistoryWriteCommon<'_>;
}

impl<I: DbWriteAccessProviderHistory> GetDbHistoryWriteCommandsCommon for I {
    fn common_history(&mut self) -> HistoryWriteCommon<'_> {
        HistoryWriteCommon::new(self.handle())
    }
}
