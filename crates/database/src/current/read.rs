use self::{
    account::{CurrentReadAccount, CurrentSyncReadAccount},
    account_admin::CurrentSyncReadAccountAdmin,
    chat::{CurrentReadChat, CurrentSyncReadChat},
    chat_admin::CurrentSyncReadChatAdmin,
    media::{CurrentReadMedia, CurrentSyncReadMedia},
    media_admin::CurrentSyncReadMediaAdmin,
    profile::{CurrentReadProfile, CurrentSyncReadProfile},
    profile_admin::CurrentSyncReadProfileAdmin, common::CurrentSyncReadCommon,
};
use crate::{
    diesel::{ConnectionProvider, DieselConnection},
    sqlite::SqlxReadHandle,
};

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

        pub struct $sync_name<C: crate::diesel::ConnectionProvider> {
            cmds: C,
        }

        impl<C: crate::diesel::ConnectionProvider> $sync_name<C> {
            pub fn new(cmds: C) -> Self {
                Self { cmds }
            }

            pub fn conn(&mut self) -> &mut crate::diesel::DieselConnection {
                self.cmds.conn()
            }
        }
    };
}

pub mod account;
pub mod account_admin;
pub mod common;
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

pub struct CurrentSyncReadCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> CurrentSyncReadCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_account(self) -> CurrentSyncReadAccount<C> {
        CurrentSyncReadAccount::new(self.conn)
    }

    pub fn into_account_admin(self) -> CurrentSyncReadAccountAdmin<C> {
        CurrentSyncReadAccountAdmin::new(self.conn)
    }

    pub fn into_media(self) -> CurrentSyncReadMedia<C> {
        CurrentSyncReadMedia::new(self.conn)
    }

    pub fn into_media_admin(self) -> CurrentSyncReadMediaAdmin<C> {
        CurrentSyncReadMediaAdmin::new(self.conn)
    }

    pub fn into_profile(self) -> CurrentSyncReadProfile<C> {
        CurrentSyncReadProfile::new(self.conn)
    }

    pub fn into_profile_admin(self) -> CurrentSyncReadProfileAdmin<C> {
        CurrentSyncReadProfileAdmin::new(self.conn)
    }

    pub fn into_chat(self) -> CurrentSyncReadChat<C> {
        CurrentSyncReadChat::new(self.conn)
    }

    pub fn into_chat_admin(self) -> CurrentSyncReadChatAdmin<C> {
        CurrentSyncReadChatAdmin::new(self.conn)
    }

    pub fn conn(&mut self) -> &mut C {
        &mut self.conn
    }
}

impl CurrentSyncReadCommands<&mut DieselConnection> {
    pub fn account(&mut self) -> CurrentSyncReadAccount<&mut DieselConnection> {
        CurrentSyncReadAccount::new(self.conn())
    }

    pub fn account_admin(&mut self) -> CurrentSyncReadAccountAdmin<&mut DieselConnection> {
        CurrentSyncReadAccountAdmin::new(self.conn())
    }

    pub fn profile(&mut self) -> CurrentSyncReadProfile<&mut DieselConnection> {
        CurrentSyncReadProfile::new(self.conn())
    }

    pub fn profile_admin(&mut self) -> CurrentSyncReadProfileAdmin<&mut DieselConnection> {
        CurrentSyncReadProfileAdmin::new(self.conn())
    }

    pub fn media(&mut self) -> CurrentSyncReadMedia<&mut DieselConnection> {
        CurrentSyncReadMedia::new(self.conn())
    }

    pub fn media_admin(&mut self) -> CurrentSyncReadMediaAdmin<&mut DieselConnection> {
        CurrentSyncReadMediaAdmin::new(self.conn())
    }

    pub fn chat(&mut self) -> CurrentSyncReadChat<&mut DieselConnection> {
        CurrentSyncReadChat::new(self.conn())
    }

    pub fn chat_admin(&mut self) -> CurrentSyncReadChatAdmin<&mut DieselConnection> {
        CurrentSyncReadChatAdmin::new(self.conn())
    }

    pub fn common(&mut self) -> CurrentSyncReadCommon<&mut DieselConnection> {
        CurrentSyncReadCommon::new(self.conn())
    }
}
