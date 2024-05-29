use database::{ConnectionProvider, DieselConnection};

use self::{
    profile::CurrentSyncReadProfile, profile_admin::CurrentSyncReadProfileAdmin,
};
pub mod profile;
pub mod profile_admin;

pub struct CurrentSyncReadCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> CurrentSyncReadCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_profile(self) -> CurrentSyncReadProfile<C> {
        CurrentSyncReadProfile::new(self.conn)
    }

    pub fn into_profile_admin(self) -> CurrentSyncReadProfileAdmin<C> {
        CurrentSyncReadProfileAdmin::new(self.conn)
    }

   pub fn conn(&mut self) -> &mut C {
        &mut self.conn
    }
}

impl CurrentSyncReadCommands<&mut DieselConnection> {
   pub fn profile(&mut self) -> CurrentSyncReadProfile<&mut DieselConnection> {
        CurrentSyncReadProfile::new(self.conn())
    }

    pub fn profile_admin(&mut self) -> CurrentSyncReadProfileAdmin<&mut DieselConnection> {
        CurrentSyncReadProfileAdmin::new(self.conn())
    }

    pub fn common(&mut self) -> database::current::read::common::CurrentSyncReadCommon<&mut DieselConnection> {
        database::current::read::common::CurrentSyncReadCommon::new(self.conn())
    }
}
