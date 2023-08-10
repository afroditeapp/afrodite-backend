use sqlx::SqlitePool;

use crate::diesel::DieselConnection;
use crate::sqlite::CurrentDataWriteHandle;

use self::account::{CurrentSyncWriteAccount, CurrentWriteAccount};
use self::chat::{CurrentSyncWriteChat, CurrentWriteChat};
use self::media::{CurrentSyncWriteMedia, CurrentWriteMedia};
use self::media_admin::CurrentWriteMediaAdmin;
use self::profile::{CurrentSyncWriteProfile, CurrentWriteProfile};

macro_rules! define_write_commands {
    ($struct_name:ident, $sync_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: &'a crate::current::write::CurrentWriteCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: &'a crate::current::write::CurrentWriteCommands<'a>) -> Self {
                Self { cmds }
            }

            pub fn read(&self) -> crate::current::read::SqliteReadCommands<'a> {
                self.cmds.handle.read()
            }

            pub fn pool(&self) -> &'a sqlx::SqlitePool {
                self.cmds.handle.pool()
            }
        }

        pub struct $sync_name<'a> {
            cmds: crate::current::write::CurrentSyncWriteCommands<'a>,
        }

        impl<'a> $sync_name<'a> {
            pub fn new(cmds: crate::current::write::CurrentSyncWriteCommands<'a>) -> Self {
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

#[derive(Clone, Debug)]
pub struct CurrentWriteCommands<'a> {
    handle: &'a CurrentDataWriteHandle,
}

impl<'a> CurrentWriteCommands<'a> {
    pub fn new(handle: &'a CurrentDataWriteHandle) -> Self {
        Self { handle }
    }

    pub fn account(&'a self) -> CurrentWriteAccount<'a> {
        CurrentWriteAccount::new(self)
    }

    pub fn media(&'a self) -> CurrentWriteMedia<'a> {
        CurrentWriteMedia::new(self)
    }

    pub fn media_admin(&'a self) -> CurrentWriteMediaAdmin<'a> {
        CurrentWriteMediaAdmin::new(self)
    }

    pub fn profile(&'a self) -> CurrentWriteProfile<'a> {
        CurrentWriteProfile::new(self)
    }

    pub fn chat(&'a self) -> CurrentWriteChat<'a> {
        CurrentWriteChat::new(self)
    }

    pub fn pool(&'a self) -> &SqlitePool {
        self.handle.pool()
    }
}

pub struct CurrentSyncWriteCommands<'a> {
    conn: &'a mut DieselConnection,
}

impl<'a> CurrentSyncWriteCommands<'a> {
    pub fn new(conn: &'a mut DieselConnection) -> Self {
        Self { conn }
    }

    pub fn account(self) -> CurrentSyncWriteAccount<'a> {
        CurrentSyncWriteAccount::new(self)
    }

    pub fn media(self) -> CurrentSyncWriteMedia<'a> {
        CurrentSyncWriteMedia::new(self)
    }

    pub fn profile(self) -> CurrentSyncWriteProfile<'a> {
        CurrentSyncWriteProfile::new(self)
    }

    pub fn chat(self) -> CurrentSyncWriteChat<'a> {
        CurrentSyncWriteChat::new(self)
    }
}
