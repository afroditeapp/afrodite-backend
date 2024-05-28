use simple_backend_database::diesel_db::{
    ConnectionProvider, DieselConnection, DieselDatabaseError,
};

use self::{
    account::CurrentSyncWriteAccount, chat::CurrentSyncWriteChat, common::CurrentSyncWriteCommon,
    media::CurrentSyncWriteMedia, media_admin::CurrentSyncWriteMediaAdmin,
    profile::CurrentSyncWriteProfile,
};
use crate::TransactionError;

macro_rules! define_write_commands {
    ($struct_name:ident, $sync_name:ident) => {
        // TODO: Remove struct_name

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

            pub fn cmds(
                &mut self,
            ) -> crate::current::write::CurrentSyncWriteCommands<
                &mut simple_backend_database::diesel_db::DieselConnection,
            > {
                crate::current::write::CurrentSyncWriteCommands::new(self.conn())
            }

            pub fn read_conn(
                conn: &mut simple_backend_database::diesel_db::DieselConnection,
            ) -> crate::current::read::CurrentSyncReadCommands<
                &mut simple_backend_database::diesel_db::DieselConnection,
            > {
                crate::current::read::CurrentSyncReadCommands::new(conn)
            }

            pub fn read(
                &mut self,
            ) -> crate::current::read::CurrentSyncReadCommands<
                &mut simple_backend_database::diesel_db::DieselConnection,
            > {
                crate::current::read::CurrentSyncReadCommands::new(self.conn())
            }
        }
    };
}

pub mod account;
pub mod account_admin;
pub mod chat;
pub mod chat_admin;
pub mod common;
pub mod media;
pub mod media_admin;
pub mod profile;
pub mod profile_admin;

pub struct CurrentSyncWriteCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> CurrentSyncWriteCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn read(&mut self) -> crate::current::read::CurrentSyncReadCommands<&mut DieselConnection> {
        crate::current::read::CurrentSyncReadCommands::new(self.conn.conn())
    }

    pub fn write(&mut self) -> &mut C {
        &mut self.conn
    }

    pub fn conn(&mut self) -> &mut DieselConnection {
        self.conn.conn()
    }
}

/// Write commands for current database. All commands must be run in
/// a database transaction.
impl CurrentSyncWriteCommands<&mut DieselConnection> {
    pub fn account(&mut self) -> CurrentSyncWriteAccount<&mut DieselConnection> {
        CurrentSyncWriteAccount::new(self.write())
    }

    pub fn common(&mut self) -> CurrentSyncWriteCommon<&mut DieselConnection> {
        CurrentSyncWriteCommon::new(self.write())
    }

    pub fn chat(&mut self) -> CurrentSyncWriteChat<&mut DieselConnection> {
        CurrentSyncWriteChat::new(self.write())
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
        ) -> std::result::Result<T, TransactionError>,
        T,
    >(
        self,
        transaction_actions: F,
    ) -> error_stack::Result<T, DieselDatabaseError> {
        use diesel::prelude::*;
        self.conn.transaction(transaction_actions)
            .map_err(|e| e.into_report())
    }
}

pub struct TransactionConnection<'a> {
    conn: &'a mut DieselConnection,
}

impl<'a> TransactionConnection<'a> {
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
