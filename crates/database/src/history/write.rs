use simple_backend_database::{
    diesel_db::{ConnectionProvider, DieselConnection, DieselDatabaseError},
    sqlx_db::SqlxWriteHandle,
};
use sqlx::SqlitePool;

// use sqlx::SqlitePool;
use self::{
    account::{HistorySyncWriteAccount, HistoryWriteAccount},
    chat::{HistorySyncWriteChat, HistoryWriteChat},
    media::{HistorySyncWriteMedia, HistoryWriteMedia},
    media_admin::{HistorySyncWriteMediaAdmin, HistoryWriteMediaAdmin},
    profile::{HistorySyncWriteProfile, HistoryWriteProfile},
};
use crate::{HistoryWriteHandle, TransactionError};

macro_rules! define_write_commands {
    ($struct_name:ident, $sync_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: &'a crate::history::write::HistoryWriteCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: &'a crate::history::write::HistoryWriteCommands<'a>) -> Self {
                Self { cmds }
            }

            // pub fn read(&self) -> crate::history::read::HistoryReadCommands<'a> {
            //     self.cmds.handle.read()
            // }

            pub fn pool(&self) -> &'a sqlx::SqlitePool {
                self.cmds.handle.pool()
            }
        }

        pub struct $sync_name<C: simple_backend_database::diesel_db::ConnectionProvider> {
            cmds: C,
        }

        impl<C: simple_backend_database::diesel_db::ConnectionProvider> $sync_name<C> {
            pub fn new(cmds: C) -> Self {
                Self { cmds }
            }

            pub fn conn(&mut self) -> &mut simple_backend_database::diesel_db::DieselConnection {
                self.cmds.conn()
            }

            // pub fn into_conn(self) -> &'a mut crate::diesel::DieselConnection {
            //     self.cmds.conn
            // }

            pub fn read(
                conn: &mut simple_backend_database::diesel_db::DieselConnection,
            ) -> crate::history::read::HistorySyncReadCommands<
                &mut simple_backend_database::diesel_db::DieselConnection,
            > {
                crate::history::read::HistorySyncReadCommands::new(conn)
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
pub struct HistoryWriteCommands<'a> {
    handle: &'a SqlxWriteHandle,
}

impl<'a> HistoryWriteCommands<'a> {
    pub fn new(handle: &'a HistoryWriteHandle) -> Self {
        Self {
            handle: handle.0.sqlx(),
        }
    }

    pub fn account(&'a self) -> HistoryWriteAccount<'a> {
        HistoryWriteAccount::new(self)
    }

    pub fn media(&'a self) -> HistoryWriteMedia<'a> {
        HistoryWriteMedia::new(self)
    }

    pub fn media_admin(&'a self) -> HistoryWriteMediaAdmin<'a> {
        HistoryWriteMediaAdmin::new(self)
    }

    pub fn profile(&'a self) -> HistoryWriteProfile<'a> {
        HistoryWriteProfile::new(self)
    }

    pub fn chat(&'a self) -> HistoryWriteChat<'a> {
        HistoryWriteChat::new(self)
    }

    pub fn pool(&'a self) -> &SqlitePool {
        self.handle.pool()
    }
}

pub struct HistorySyncWriteCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> HistorySyncWriteCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_account(self) -> HistorySyncWriteAccount<C> {
        HistorySyncWriteAccount::new(self.conn)
    }

    pub fn into_media(self) -> HistorySyncWriteMedia<C> {
        HistorySyncWriteMedia::new(self.conn)
    }

    pub fn into_media_admin(self) -> HistorySyncWriteMediaAdmin<C> {
        HistorySyncWriteMediaAdmin::new(self.conn)
    }

    pub fn into_profile(self) -> HistorySyncWriteProfile<C> {
        HistorySyncWriteProfile::new(self.conn)
    }

    pub fn into_chat(self) -> HistorySyncWriteChat<C> {
        HistorySyncWriteChat::new(self.conn)
    }

    // pub fn read(&mut self) -> crate::history::read::HistorySyncReadCommands<&mut DieselConnection> {
    //     self.conn.read()
    // }

    pub fn write(&mut self) -> &mut C {
        &mut self.conn
    }

    pub fn conn(&mut self) -> &mut DieselConnection {
        self.conn.conn()
    }
}

impl HistorySyncWriteCommands<&mut DieselConnection> {
    pub fn account(&mut self) -> HistorySyncWriteAccount<&mut DieselConnection> {
        HistorySyncWriteAccount::new(self.write())
    }

    pub fn media(&mut self) -> HistorySyncWriteMedia<&mut DieselConnection> {
        HistorySyncWriteMedia::new(self.write())
    }

    pub fn media_admin(&mut self) -> HistorySyncWriteMediaAdmin<&mut DieselConnection> {
        HistorySyncWriteMediaAdmin::new(self.write())
    }

    pub fn profile(&mut self) -> HistorySyncWriteProfile<&mut DieselConnection> {
        HistorySyncWriteProfile::new(self.write())
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
