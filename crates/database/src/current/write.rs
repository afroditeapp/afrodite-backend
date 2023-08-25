use sqlx::SqlitePool;

use self::{
    account::{CurrentSyncWriteAccount, CurrentWriteAccount},
    chat::{CurrentSyncWriteChat, CurrentWriteChat},
    media::{CurrentSyncWriteMedia, CurrentWriteMedia},
    media_admin::{CurrentSyncWriteMediaAdmin, CurrentWriteMediaAdmin},
    profile::{CurrentSyncWriteProfile, CurrentWriteProfile},
};
use crate::{
    diesel::{DieselConnection, DieselDatabaseError, ConnectionProvider},
    sqlite::CurrentDataWriteHandle,
    TransactionError,
};

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

            // pub fn into_conn(self) -> &'a mut crate::diesel::DieselConnection {
            //     self.cmds.conn
            // }

            pub fn cmds(&mut self) -> crate::current::write::CurrentSyncWriteCommands<&mut crate::diesel::DieselConnection> {
                crate::current::write::CurrentSyncWriteCommands::new(self.conn())
            }

            pub fn read(
                conn: &mut crate::diesel::DieselConnection,
            ) -> crate::current::read::CurrentSyncReadCommands<&mut crate::diesel::DieselConnection> {
                crate::current::read::CurrentSyncReadCommands::new(conn)
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

pub struct CurrentSyncWriteCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> CurrentSyncWriteCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_account(self) -> CurrentSyncWriteAccount<C> {
        CurrentSyncWriteAccount::new(self.conn)
    }

    pub fn into_media(self) -> CurrentSyncWriteMedia<C> {
        CurrentSyncWriteMedia::new(self.conn)
    }

    pub fn into_media_admin(self) -> CurrentSyncWriteMediaAdmin<C> {
        CurrentSyncWriteMediaAdmin::new(self.conn)
    }

    pub fn into_profile(self) -> CurrentSyncWriteProfile<C> {
        CurrentSyncWriteProfile::new(self.conn)
    }

    pub fn into_chat(self) -> CurrentSyncWriteChat<C> {
        CurrentSyncWriteChat::new(self.conn)
    }

    pub fn read(&mut self) -> crate::current::read::CurrentSyncReadCommands<&mut crate::diesel::DieselConnection> {
        self.conn.read()
    }

    pub fn write(&mut self) -> &mut C {
        &mut self.conn
    }

    pub fn conn(&mut self) -> &mut DieselConnection {
        self.conn.conn()
    }
}

impl CurrentSyncWriteCommands<&mut DieselConnection> {
    pub fn account(&mut self) -> CurrentSyncWriteAccount<&mut DieselConnection> {
        CurrentSyncWriteAccount::new(self.write())
    }

    pub fn media(&mut self) -> CurrentSyncWriteMedia<&mut DieselConnection> {
        CurrentSyncWriteMedia::new(self.write())
    }

    pub fn media_admin(&mut self) -> CurrentSyncWriteMediaAdmin<&mut DieselConnection> {
        CurrentSyncWriteMediaAdmin::new(self.write())
    }

    pub fn profile(&mut self) -> CurrentSyncWriteProfile<&mut DieselConnection> {
        CurrentSyncWriteProfile::new(self.write())
    }

    pub fn transaction<
        F: FnOnce(
                &mut DieselConnection,
            ) -> std::result::Result<T, TransactionError<DieselDatabaseError>>
            + 'static,
        T,
    >(
        self,
        transaction_actions: F,
    ) -> error_stack::Result<T, DieselDatabaseError> {
        use diesel::prelude::*;
        Ok(self.conn.transaction(transaction_actions)?)
    }
}

pub struct TransactionConnection<'a> {
    conn: &'a mut DieselConnection,
}

impl <'a> TransactionConnection<'a> {
    pub fn new(conn: &'a mut DieselConnection) -> Self {
        Self { conn }
    }

    pub fn into_conn(self) -> &'a mut DieselConnection {
        self.conn
    }

    pub fn into_cmds(self) -> CurrentSyncWriteCommands<&'a mut DieselConnection> {
        CurrentSyncWriteCommands::new(self.conn)
    }
}

impl ConnectionProvider for &mut TransactionConnection<'_> {
    fn conn(&mut self) -> &mut DieselConnection {
        self.conn
    }
}
