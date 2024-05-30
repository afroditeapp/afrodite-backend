use database::ConnectionProvider;

use self::{media::HistorySyncReadMedia, media_admin::HistorySyncReadMediaAdmin};

pub mod media;
pub mod media_admin;

pub struct HistorySyncReadCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> HistorySyncReadCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_media(self) -> HistorySyncReadMedia<C> {
        HistorySyncReadMedia::new(self.conn)
    }

    pub fn into_media_admin(self) -> HistorySyncReadMediaAdmin<C> {
        HistorySyncReadMediaAdmin::new(self.conn)
    }
}
