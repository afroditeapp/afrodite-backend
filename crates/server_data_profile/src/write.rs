//! Synchronous write commands combining cache and database operations.

use profile::WriteCommandsProfile;
use profile_admin::WriteCommandsProfileAdmin;
use server_data::write::WriteCommands;

pub mod profile;
pub mod profile_admin;

pub trait GetWriteCommandsProfile<'a>: Sized {
    fn profile(self) -> WriteCommandsProfile<'a>;
    fn profile_admin(self) -> WriteCommandsProfileAdmin<'a>;
}

impl <'a> GetWriteCommandsProfile<'a> for WriteCommands<'a> {
    fn profile(self) -> WriteCommandsProfile<'a> {
        WriteCommandsProfile::new(self)
    }

    fn profile_admin(self) -> WriteCommandsProfileAdmin<'a> {
        WriteCommandsProfileAdmin::new(self)
    }
}
