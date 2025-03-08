use common::HistoryReadCommon;

use crate::DbReadAccessProviderHistory;

pub mod common;

pub trait GetDbHistoryReadCommandsCommon {
    fn common_history(&mut self) -> HistoryReadCommon<'_>;
}

impl<I: DbReadAccessProviderHistory> GetDbHistoryReadCommandsCommon for I {
    fn common_history(&mut self) -> HistoryReadCommon<'_> {
        HistoryReadCommon::new(self.handle())
    }
}
