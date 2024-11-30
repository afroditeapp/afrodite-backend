//! Synchronous write commands combining cache and database operations.

use profile::WriteCommandsProfile;
use profile_admin::WriteCommandsProfileAdmin;
use profile_admin_history::WriteCommandsProfileAdminHistory;
use server_data::db_manager::WriteAccessProvider;

pub mod profile;
pub mod profile_admin;
pub mod profile_admin_history;

pub trait GetWriteCommandsProfile<'a> {
    fn profile(self) -> WriteCommandsProfile<'a>;
    fn profile_admin(self) -> WriteCommandsProfileAdmin<'a>;
    fn profile_admin_history(self) -> WriteCommandsProfileAdminHistory<'a>;
}

impl<'a, I: WriteAccessProvider<'a>> GetWriteCommandsProfile<'a> for I {
    fn profile(self) -> WriteCommandsProfile<'a> {
        WriteCommandsProfile::new(self.handle())
    }

    fn profile_admin(self) -> WriteCommandsProfileAdmin<'a> {
        WriteCommandsProfileAdmin::new(self.handle())
    }

    fn profile_admin_history(self) -> WriteCommandsProfileAdminHistory<'a> {
        WriteCommandsProfileAdminHistory::new(self.handle())
    }
}
