use profile::ReadCommandsProfile;
use profile_admin::ReadCommandsProfileAdmin;
use server_data::read::ReadCommands;

pub mod profile;
pub mod profile_admin;

pub trait GetReadProfileCommands<'a>: Sized {
    fn profile(self) -> ReadCommandsProfile<'a>;
    fn profile_admin(self) -> ReadCommandsProfileAdmin<'a>;
}

impl <'a> GetReadProfileCommands<'a> for ReadCommands<'a> {
    fn profile(self) -> ReadCommandsProfile<'a> {
        ReadCommandsProfile::new(self)
    }

    fn profile_admin(self) -> ReadCommandsProfileAdmin<'a> {
        ReadCommandsProfileAdmin::new(self)
    }
}
