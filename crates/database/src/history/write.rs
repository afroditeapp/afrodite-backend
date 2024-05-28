use simple_backend_database::diesel_db::{ConnectionProvider, DieselConnection, DieselDatabaseError};

use self::{
    account::HistorySyncWriteAccount,
    chat::HistorySyncWriteChat,
    media::HistorySyncWriteMedia,
    media_admin::HistorySyncWriteMediaAdmin,
    profile::HistorySyncWriteProfile,
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
