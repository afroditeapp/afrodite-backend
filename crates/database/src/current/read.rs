use common::CurrentSyncReadCommon;
use simple_backend_database::diesel_db::{ConnectionProvider, DieselConnection};

macro_rules! define_read_commands {
    ($struct_name:ident, $sync_name:ident) => {
        // TODO: Remove struct_name

        pub struct $sync_name<C: simple_backend_database::diesel_db::ConnectionProvider> {
            cmds: C,
        }

        impl<C: simple_backend_database::diesel_db::ConnectionProvider> $sync_name<C> {
            pub fn new(cmds: C) -> Self {
                Self { cmds }
            }

            #[allow(dead_code)]
            fn read(
                &mut self,
            ) -> crate::current::read::CurrentSyncReadCommands<
                &mut simple_backend_database::diesel_db::DieselConnection,
            > {
                crate::current::read::CurrentSyncReadCommands::new(self.conn())
            }

            pub fn conn(&mut self) -> &mut simple_backend_database::diesel_db::DieselConnection {
                self.cmds.conn()
            }
        }
    };
}

pub mod common;

pub struct CurrentSyncReadCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> CurrentSyncReadCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn conn(&mut self) -> &mut C {
        &mut self.conn
    }
}

impl CurrentSyncReadCommands<&mut DieselConnection> {
    pub fn common(&mut self) -> CurrentSyncReadCommon<&mut DieselConnection> {
        CurrentSyncReadCommon::new(self.conn())
    }
}
