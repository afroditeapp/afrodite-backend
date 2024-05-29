//! Synchronous write commands combining cache and database operations.


use media::WriteCommandsMedia;
use media_admin::WriteCommandsMediaAdmin;
use server_data::write::{WriteCommands, WriteCommandsProvider};

pub mod media;
pub mod media_admin;


pub trait GetWriteCommandsMedia<C: WriteCommandsProvider> {
    fn media(self) -> WriteCommandsMedia<C>;
    fn media_admin(self) -> WriteCommandsMediaAdmin<C>;
}

impl <C: WriteCommandsProvider> GetWriteCommandsMedia<C> for C {
    fn media(self) -> WriteCommandsMedia<C> {
        WriteCommandsMedia::new(self)
    }

    fn media_admin(self) -> WriteCommandsMediaAdmin<C> {
        WriteCommandsMediaAdmin::new(self)
    }
}
