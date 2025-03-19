//! Synchronous write commands combining cache and database operations.

use profile::WriteCommandsProfile;
use profile_admin::WriteCommandsProfileAdmin;
use profile_admin_history::WriteCommandsProfileAdminHistory;
use server_data::db_manager::WriteAccessProvider;

pub mod profile;
pub mod profile_admin;
pub mod profile_admin_history;

pub trait GetWriteCommandsProfile {
    fn profile(&self) -> WriteCommandsProfile<'_>;
    fn profile_admin(&self) -> WriteCommandsProfileAdmin<'_>;
    fn profile_admin_history(&self) -> WriteCommandsProfileAdminHistory<'_>;
}

impl<I: WriteAccessProvider> GetWriteCommandsProfile for I {
    fn profile(&self) -> WriteCommandsProfile<'_> {
        WriteCommandsProfile::new(self.handle())
    }

    fn profile_admin(&self) -> WriteCommandsProfileAdmin<'_> {
        WriteCommandsProfileAdmin::new(self.handle())
    }

    fn profile_admin_history(&self) -> WriteCommandsProfileAdminHistory<'_> {
        WriteCommandsProfileAdminHistory::new(self.handle())
    }
}
