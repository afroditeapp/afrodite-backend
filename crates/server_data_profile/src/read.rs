use profile::ReadCommandsProfile;
use profile_admin::ReadCommandsProfileAdmin;
use server_data::read::{ReadCommands, ReadCommandsProvider};

pub mod profile;
pub mod profile_admin;

pub trait GetReadProfileCommands<C: ReadCommandsProvider> {
    fn profile(self) -> ReadCommandsProfile<C>;
    fn profile_admin(self) -> ReadCommandsProfileAdmin<C>;
}

impl <C: ReadCommandsProvider> GetReadProfileCommands<C> for C {
    fn profile(self) -> ReadCommandsProfile<C> {
        ReadCommandsProfile::new(self)
    }

    fn profile_admin(self) -> ReadCommandsProfileAdmin<C> {
        ReadCommandsProfileAdmin::new(self)
    }
}
