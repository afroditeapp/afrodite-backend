//! Synchronous write commands combining cache and database operations.

use profile::WriteCommandsProfile;
use profile_admin::WriteCommandsProfileAdmin;
use server_data::write::{WriteCommands, WriteCommandsProvider};

pub mod profile;
pub mod profile_admin;

pub trait GetWriteCommandsProfile<C: WriteCommandsProvider> {
    fn profile(self) -> WriteCommandsProfile<C>;
    fn profile_admin(self) -> WriteCommandsProfileAdmin<C>;
}

impl <C: WriteCommandsProvider> GetWriteCommandsProfile<C> for C {
    fn profile(self) -> WriteCommandsProfile<C> {
        WriteCommandsProfile::new(self)
    }

    fn profile_admin(self) -> WriteCommandsProfileAdmin<C> {
        WriteCommandsProfileAdmin::new(self)
    }
}
