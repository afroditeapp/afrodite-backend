//! Synchronous write commands combining cache and database operations.

use profile::WriteCommandsProfile;
use profile_admin::WriteCommandsProfileAdmin;
use profile_admin_history::WriteCommandsProfileAdminHistory;
use server_data::write::WriteCommandsProvider;

pub mod profile;
pub mod profile_admin;
pub mod profile_admin_history;

pub trait GetWriteCommandsProfile<C: WriteCommandsProvider> {
    fn profile(self) -> WriteCommandsProfile<C>;
    fn profile_admin(self) -> WriteCommandsProfileAdmin<C>;
    fn profile_admin_history(self) -> WriteCommandsProfileAdminHistory<C>;
}

impl<C: WriteCommandsProvider> GetWriteCommandsProfile<C> for C {
    fn profile(self) -> WriteCommandsProfile<C> {
        WriteCommandsProfile::new(self)
    }

    fn profile_admin(self) -> WriteCommandsProfileAdmin<C> {
        WriteCommandsProfileAdmin::new(self)
    }

    fn profile_admin_history(self) -> WriteCommandsProfileAdminHistory<C> {
        WriteCommandsProfileAdminHistory::new(self)
    }
}
