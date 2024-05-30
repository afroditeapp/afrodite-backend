use database::{ConnectionProvider, DieselConnection};

use self::{media::CurrentSyncReadMedia, media_admin::CurrentSyncReadMediaAdmin};

pub mod media;
pub mod media_admin;

pub struct CurrentSyncReadCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> CurrentSyncReadCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_media(self) -> CurrentSyncReadMedia<C> {
        CurrentSyncReadMedia::new(self.conn)
    }

    pub fn into_media_admin(self) -> CurrentSyncReadMediaAdmin<C> {
        CurrentSyncReadMediaAdmin::new(self.conn)
    }

    pub fn conn(&mut self) -> &mut C {
        &mut self.conn
    }
}

impl CurrentSyncReadCommands<&mut DieselConnection> {
    pub fn media(&mut self) -> CurrentSyncReadMedia<&mut DieselConnection> {
        CurrentSyncReadMedia::new(self.conn())
    }

    pub fn media_admin(&mut self) -> CurrentSyncReadMediaAdmin<&mut DieselConnection> {
        CurrentSyncReadMediaAdmin::new(self.conn())
    }

    pub fn common(
        &mut self,
    ) -> database::current::read::common::CurrentSyncReadCommon<&mut DieselConnection> {
        database::current::read::common::CurrentSyncReadCommon::new(self.conn())
    }
}
