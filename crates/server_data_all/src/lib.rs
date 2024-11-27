#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use database::{ConnectionProvider, DieselConnection};
use database_account::current::{read::account::CurrentSyncReadAccount, write::account::CurrentSyncWriteAccount};
use database_chat::current::{read::chat::CurrentSyncReadChat, write::chat::CurrentSyncWriteChat};
use database_media::current::{read::media::CurrentSyncReadMedia, write::media::CurrentSyncWriteMedia};
use database_profile::current::{read::profile::CurrentSyncReadProfile, write::profile::CurrentSyncWriteProfile};


pub struct CurrentSyncReadCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> CurrentSyncReadCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn read(&mut self) -> &mut C {
        &mut self.conn
    }

    pub fn conn(&mut self) -> &mut DieselConnection {
        self.conn.conn()
    }
}

impl CurrentSyncReadCommands<&mut DieselConnection> {
    pub fn account(&mut self) -> CurrentSyncReadAccount<&mut DieselConnection> {
        CurrentSyncReadAccount::new(self.read())
    }

    pub fn profile(&mut self) -> CurrentSyncReadProfile<&mut DieselConnection> {
        CurrentSyncReadProfile::new(self.read())
    }

    pub fn chat(&mut self) -> CurrentSyncReadChat<&mut DieselConnection> {
        CurrentSyncReadChat::new(self.read())
    }

    pub fn media(&mut self) -> CurrentSyncReadMedia<&mut DieselConnection> {
        CurrentSyncReadMedia::new(self.read())
    }

    pub fn common(
        &mut self,
    ) -> database::current::read::common::CurrentSyncReadCommon<&mut DieselConnection> {
        database::current::read::common::CurrentSyncReadCommon::new(self.read())
    }
}


pub struct CurrentSyncWriteCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> CurrentSyncWriteCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
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

    pub fn profile(&mut self) -> CurrentSyncWriteProfile<&mut DieselConnection> {
        CurrentSyncWriteProfile::new(self.write())
    }

    pub fn chat(&mut self) -> CurrentSyncWriteChat<&mut DieselConnection> {
        CurrentSyncWriteChat::new(self.write())
    }

    pub fn media(&mut self) -> CurrentSyncWriteMedia<&mut DieselConnection> {
        CurrentSyncWriteMedia::new(self.write())
    }

    pub fn common(
        &mut self,
    ) -> database::current::write::common::CurrentSyncWriteCommon<&mut DieselConnection> {
        database::current::write::common::CurrentSyncWriteCommon::new(self.write())
    }

    pub fn read(&mut self) -> CurrentSyncReadCommands<&mut DieselConnection> {
        CurrentSyncReadCommands::new(self.write())
    }
}

macro_rules! db_transaction {
    ($state:expr, move |mut $cmds:ident| $commands:expr) => {{
        server_common::data::IntoDataError::into_error(
            $state.db_transaction(move |mut $cmds| ($commands)).await,
        )
    }};
    ($state:expr, move |$cmds:ident| $commands:expr) => {{
        $crate::data::IntoDataError::into_error(
            $state.db_transaction(move |$cmds| ($commands)).await,
        )
    }};
}

pub mod demo;
pub mod load;
pub mod register;
pub mod unlimited_likes;
