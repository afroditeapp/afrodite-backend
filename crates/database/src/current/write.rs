use common_admin::CurrentWriteCommonAdmin;

use self::common::CurrentWriteCommon;
use crate::{DbWriteAccessProvider, DbWriteMode};

pub mod common;
pub mod common_admin;

pub trait GetDbWriteCommandsCommon {
    fn common(&mut self) -> CurrentWriteCommon<'_>;
    fn common_admin(&mut self) -> CurrentWriteCommonAdmin<'_>;
}

impl<I: DbWriteAccessProvider> GetDbWriteCommandsCommon for I {
    fn common(&mut self) -> CurrentWriteCommon<'_> {
        CurrentWriteCommon::new(self.handle())
    }

    fn common_admin(&mut self) -> CurrentWriteCommonAdmin<'_> {
        CurrentWriteCommonAdmin::new(self.handle())
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
