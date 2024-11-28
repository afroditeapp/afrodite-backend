use self::common::CurrentWriteCommon;
use crate::{DbWriteAccessProvider, DbWriteMode};

pub mod common;

pub trait GetDbWriteCommandsCommon {
    fn common(&mut self) -> CurrentWriteCommon<'_>;
}

impl <I: DbWriteAccessProvider> GetDbWriteCommandsCommon for I {
    fn common(&mut self) -> CurrentWriteCommon<'_> {
        CurrentWriteCommon::new(self.handle())
    }
}

pub struct TransactionConnection<'a> {
    conn: DbWriteMode<'a>,
}

impl<'a> TransactionConnection<'a> {
    pub fn new(conn: DbWriteMode<'a>) -> Self {
        Self { conn }
    }

    pub fn into_conn(self) -> DbWriteMode<'a> {
        self.conn
    }
}
