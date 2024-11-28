use common::CurrentReadCommon;

use crate::DbReadAccessProvider;

pub mod common;

pub trait GetDbReadCommandsCommon {
    fn common(&mut self) -> CurrentReadCommon<'_>;
}

impl <I: DbReadAccessProvider> GetDbReadCommandsCommon for I {
    fn common(&mut self) -> CurrentReadCommon<'_> {
        CurrentReadCommon::new(self.handle())
    }
}
