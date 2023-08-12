use self::{
    account::{CurrentReadAccount, CurrentSyncReadAccount},
    chat::{CurrentReadChat, CurrentSyncReadChat},
    media::{CurrentReadMedia, CurrentSyncReadMedia},
    profile::{CurrentReadProfile, CurrentSyncReadProfile},
};
use crate::{diesel::DieselConnection, sqlite::SqlxReadHandle};

macro_rules! define_read_commands {
    ($struct_name:ident, $sync_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: &'a crate::current::read::SqliteReadCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: &'a crate::current::read::SqliteReadCommands<'a>) -> Self {
                Self { cmds }
            }

            pub fn pool(&self) -> &'a sqlx::SqlitePool {
                self.cmds.handle.pool()
            }
        }

        pub struct $sync_name<'a> {
            cmds: crate::current::read::CurrentSyncReadCommands<'a>,
        }

        impl<'a> $sync_name<'a> {
            pub fn new(cmds: crate::current::read::CurrentSyncReadCommands<'a>) -> Self {
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

pub struct SqliteReadCommands<'a> {
    pub handle: &'a SqlxReadHandle,
}

impl<'a> SqliteReadCommands<'a> {
    pub fn new(handle: &'a SqlxReadHandle) -> Self {
        Self { handle }
    }

    pub fn account(&self) -> CurrentReadAccount<'_> {
        CurrentReadAccount::new(self)
    }

    pub fn media(&self) -> CurrentReadMedia<'_> {
        CurrentReadMedia::new(self)
    }

    pub fn profile(&self) -> CurrentReadProfile<'_> {
        CurrentReadProfile::new(self)
    }

    pub fn chat(&self) -> CurrentReadChat<'_> {
        CurrentReadChat::new(self)
    }

    pub fn pool(&self) -> &'a sqlx::SqlitePool {
        self.handle.pool()
    }
}

pub struct CurrentSyncReadCommands<'a> {
    pub conn: &'a mut DieselConnection,
}

impl<'a> CurrentSyncReadCommands<'a> {
    pub fn new(conn: &'a mut DieselConnection) -> Self {
        Self { conn }
    }

    pub fn account(self) -> CurrentSyncReadAccount<'a> {
        CurrentSyncReadAccount::new(self)
    }

    pub fn media(self) -> CurrentSyncReadMedia<'a> {
        CurrentSyncReadMedia::new(self)
    }

    pub fn profile(self) -> CurrentSyncReadProfile<'a> {
        CurrentSyncReadProfile::new(self)
    }

    pub fn chat(self) -> CurrentSyncReadChat<'a> {
        CurrentSyncReadChat::new(self)
    }
}
