use crate::server::data::database::{sqlite::{SqlxReadHandle}, diesel::DieselConnection};

use self::{account::{CurrentReadAccount, CurrentSyncReadAccount}, media::{CurrentReadMedia, CurrentSyncReadMedia}, profile::{CurrentReadProfile, CurrentSyncReadProfile}, chat::{CurrentReadChat, CurrentSyncReadChat}};

macro_rules! define_read_commands {
    ($struct_name:ident, $sync_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: &'a crate::server::data::database::current::read::SqliteReadCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: &'a crate::server::data::database::current::read::SqliteReadCommands<'a>) -> Self {
                Self { cmds }
            }

            pub fn pool(&self) -> &'a sqlx::SqlitePool {
                self.cmds.handle.pool()
            }
        }

        pub struct $sync_name<'a> {
            cmds: &'a mut crate::server::data::database::current::read::CurrentSyncReadCommands<'a>,
        }

        impl<'a> $sync_name<'a> {
            pub fn new(cmds: &'a mut crate::server::data::database::current::read::CurrentSyncReadCommands<'a>) -> Self {
                Self { cmds }
            }

            pub fn conn(&'a mut self) -> &'a mut crate::server::data::database::diesel::DieselConnection {
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
        Self {
            handle,
        }
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
        Self {
            conn,
        }
    }

    pub fn account(&'a mut self) -> CurrentSyncReadAccount<'a> {
        CurrentSyncReadAccount::new(self)
    }

    pub fn media(&'a mut self) -> CurrentSyncReadMedia<'a> {
        CurrentSyncReadMedia::new(self)
    }

    pub fn profile(&'a mut self) -> CurrentSyncReadProfile<'a> {
        CurrentSyncReadProfile::new(self)
    }

    pub fn chat(&'a mut self) -> CurrentSyncReadChat<'a> {
        CurrentSyncReadChat::new(self)
    }
}
