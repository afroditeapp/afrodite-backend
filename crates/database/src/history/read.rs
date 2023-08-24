use error_stack::Result;
use model::{AccountIdInternal, AccountState};
use time::OffsetDateTime;
use tokio_stream::{Stream, StreamExt};
use utils::IntoReportExt;

use super::{
    super::sqlite::{SqliteDatabaseError},
    HistoryData,
};

use self::{
    account::{HistoryReadAccount, HistorySyncReadAccount},
    account_admin::HistorySyncReadAccountAdmin,
    chat::{HistoryReadChat, HistorySyncReadChat},
    chat_admin::HistorySyncReadChatAdmin,
    media::{HistoryReadMedia, HistorySyncReadMedia},
    media_admin::HistorySyncReadMediaAdmin,
    profile::{HistoryReadProfile, HistorySyncReadProfile},
    profile_admin::HistorySyncReadProfileAdmin,
};

use crate::{diesel::DieselConnection, sqlite::SqlxReadHandle};

macro_rules! define_read_commands {
    ($struct_name:ident, $sync_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: &'a crate::history::read::HistoryReadCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: &'a crate::history::read::HistoryReadCommands<'a>) -> Self {
                Self { cmds }
            }

            pub fn pool(&self) -> &'a sqlx::SqlitePool {
                self.cmds.handle.pool()
            }
        }

        pub struct $sync_name<'a> {
            cmds: crate::history::read::HistorySyncReadCommands<'a>,
        }

        impl<'a> $sync_name<'a> {
            pub fn new(cmds: crate::history::read::HistorySyncReadCommands<'a>) -> Self {
                Self { cmds }
            }

            pub fn conn(&'a mut self) -> &'a mut crate::diesel::DieselConnection {
                &mut self.cmds.conn
            }
        }
    };
}

pub mod account;
pub mod account_admin;
pub mod chat;
pub mod chat_admin;
pub mod media;
pub mod media_admin;
pub mod profile;
pub mod profile_admin;


pub struct HistoryReadCommands<'a> {
    handle: &'a SqlxReadHandle,
}

impl<'a> HistoryReadCommands<'a> {
    pub fn new(handle: &'a SqlxReadHandle) -> Self {
        Self { handle }
    }
}

pub struct HistorySyncReadCommands<'a> {
    conn: &'a mut DieselConnection,
}

impl<'a> HistorySyncReadCommands<'a> {
    pub fn new(conn: &'a mut DieselConnection) -> Self {
        Self { conn }
    }

    pub fn account(self) -> HistorySyncReadAccount<'a> {
        HistorySyncReadAccount::new(self)
    }

    pub fn account_admin(self) -> HistorySyncReadAccountAdmin<'a> {
        HistorySyncReadAccountAdmin::new(self)
    }

    pub fn media(self) -> HistorySyncReadMedia<'a> {
        HistorySyncReadMedia::new(self)
    }

    pub fn media_admin(self) -> HistorySyncReadMediaAdmin<'a> {
        HistorySyncReadMediaAdmin::new(self)
    }

    pub fn profile(self) -> HistorySyncReadProfile<'a> {
        HistorySyncReadProfile::new(self)
    }

    pub fn profile_admin(self) -> HistorySyncReadProfileAdmin<'a> {
        HistorySyncReadProfileAdmin::new(self)
    }

    pub fn chat(self) -> HistorySyncReadChat<'a> {
        HistorySyncReadChat::new(self)
    }

    pub fn chat_admin(self) -> HistorySyncReadChatAdmin<'a> {
        HistorySyncReadChatAdmin::new(self)
    }
}
