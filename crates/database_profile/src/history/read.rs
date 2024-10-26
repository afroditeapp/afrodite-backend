use database::{ConnectionProvider, DieselConnection};

use self::{profile::HistorySyncReadProfile, profile_admin::HistorySyncReadProfileAdmin};

pub mod profile;
pub mod profile_admin;

pub struct HistorySyncReadCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> HistorySyncReadCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_profile(self) -> HistorySyncReadProfile<C> {
        HistorySyncReadProfile::new(self.conn)
    }

    pub fn into_profile_admin(self) -> HistorySyncReadProfileAdmin<C> {
        HistorySyncReadProfileAdmin::new(self.conn)
    }

    pub fn conn(&mut self) -> &mut C {
        &mut self.conn
    }
}

impl HistorySyncReadCommands<&mut DieselConnection> {
    pub fn profile_admin(&mut self) -> HistorySyncReadProfileAdmin<&mut DieselConnection> {
        HistorySyncReadProfileAdmin::new(self.conn())
    }
}
