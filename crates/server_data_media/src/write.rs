//! Synchronous write commands combining cache and database operations.


use media::WriteCommandsMedia;
use media_admin::WriteCommandsMediaAdmin;
use server_data::write::WriteCommands;

pub mod media;
pub mod media_admin;


pub trait GetWriteCommandsMedia<'a>: Sized {
    fn media(self) -> WriteCommandsMedia<'a>;
    fn media_admin(self) -> WriteCommandsMediaAdmin<'a>;
}

impl <'a> GetWriteCommandsMedia<'a> for WriteCommands<'a> {
    fn media(self) -> WriteCommandsMedia<'a> {
        WriteCommandsMedia::new(self)
    }

    fn media_admin(self) -> WriteCommandsMediaAdmin<'a> {
        WriteCommandsMediaAdmin::new(self)
    }
}
