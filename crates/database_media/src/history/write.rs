use database::{ConnectionProvider, DieselConnection, DieselDatabaseError, TransactionError};

use self::{media::HistorySyncWriteMedia, media_admin::HistorySyncWriteMediaAdmin};

pub mod media;
pub mod media_admin;

pub struct HistorySyncWriteCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> HistorySyncWriteCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_media(self) -> HistorySyncWriteMedia<C> {
        HistorySyncWriteMedia::new(self.conn)
    }

    pub fn into_media_admin(self) -> HistorySyncWriteMediaAdmin<C> {
        HistorySyncWriteMediaAdmin::new(self.conn)
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
    pub fn media(&mut self) -> HistorySyncWriteMedia<&mut DieselConnection> {
        HistorySyncWriteMedia::new(self.write())
    }

    pub fn media_admin(&mut self) -> HistorySyncWriteMediaAdmin<&mut DieselConnection> {
        HistorySyncWriteMediaAdmin::new(self.write())
    }

    pub fn transaction<
        F: FnOnce(&mut DieselConnection) -> std::result::Result<T, TransactionError> + 'static,
        T,
    >(
        self,
        transaction_actions: F,
    ) -> error_stack::Result<T, DieselDatabaseError> {
        use diesel::prelude::*;
        self.conn
            .transaction(transaction_actions)
            .map_err(|e| e.into_report())
    }
}
