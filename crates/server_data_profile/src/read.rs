use profile::ReadCommandsProfile;
use profile_admin::ReadCommandsProfileAdmin;
use profile_admin_history::ReadCommandsProfileAdminHistory;
use server_data::read::ReadCommandsProvider;

pub mod profile;
pub mod profile_admin;
pub mod profile_admin_history;

pub trait GetReadProfileCommands<C: ReadCommandsProvider> {
    fn profile(self) -> ReadCommandsProfile<C>;
    fn profile_admin(self) -> ReadCommandsProfileAdmin<C>;
    fn profile_admin_history(self) -> ReadCommandsProfileAdminHistory<C>;
}

impl<C: ReadCommandsProvider> GetReadProfileCommands<C> for C {
    fn profile(self) -> ReadCommandsProfile<C> {
        ReadCommandsProfile::new(self)
    }

    fn profile_admin(self) -> ReadCommandsProfileAdmin<C> {
        ReadCommandsProfileAdmin::new(self)
    }

    fn profile_admin_history(self) -> ReadCommandsProfileAdminHistory<C> {
        ReadCommandsProfileAdminHistory::new(self)
    }
}
