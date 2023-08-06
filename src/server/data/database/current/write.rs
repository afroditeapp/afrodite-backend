use sqlx::SqlitePool;

use crate::server::data::database::diesel::{DieselWriteHandle, DieselCurrentWriteHandle};
use crate::server::data::database::{diesel::DieselConnection, sqlite::SqlxReadHandle};
use crate::server::data::database::sqlite::CurrentDataWriteHandle;

use self::account::CurrentWriteAccount;
use self::chat::CurrentWriteChat;
use self::media::CurrentWriteMedia;
use self::media_admin::CurrentWriteMediaAdmin;
use self::profile::CurrentWriteProfile;

macro_rules! define_write_commands {
    ($struct_name:ident, $sync_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: &'a crate::server::data::database::current::write::CurrentWriteCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: &'a crate::server::data::database::current::write::CurrentWriteCommands<'a>) -> Self {
                Self { cmds }
            }

            pub fn read(&self) -> crate::server::data::database::current::read::SqliteReadCommands<'a> {
                self.cmds.handle.read()
            }

            pub fn pool(&self) -> &'a sqlx::SqlitePool {
                self.cmds.handle.pool()
            }
        }

        pub struct $sync_name<'a> {
            cmds: &'a mut crate::server::data::database::current::write::CurrentSyncWriteCommands<'a>,
        }

        impl<'a> $sync_name<'a> {
            pub fn new(cmds: &'a mut crate::server::data::database::current::write::CurrentSyncWriteCommands<'a>) -> Self {
                Self { cmds }
            }

            pub fn conn(&'a mut self) -> &'a mut crate::server::data::database::diesel::DieselConnection {
                unimplemented!()
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
    handle: &'a DieselCurrentWriteHandle,
}

// impl<'a> CurrentDataWriteCommands<'a> {
//     pub fn new(handle: &'a CurrentDataWriteHandle) -> Self {
//         Self { handle }
//     }

//     pub fn account(self) -> CurrentWriteAccountCommands<'a> {
//         CurrentWriteAccountCommands::new(self.handle)
//     }

//     pub fn media(self) -> CurrentWriteMediaCommands<'a> {
//         CurrentWriteMediaCommands::new(self.handle)
//     }

//     pub fn media_admin(self) -> CurrentWriteMediaAdminCommands<'a> {
//         CurrentWriteMediaAdminCommands::new(self.handle)
//     }

//     pub fn profile(self) -> CurrentWriteProfileCommands<'a> {
//         CurrentWriteProfileCommands::new(self.handle)
//     }

//     pub fn chat(self) -> CurrentWriteChatCommands<'a> {
//         CurrentWriteChatCommands::new(self.handle)
//     }
// }
