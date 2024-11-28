//! Synchronous write commands combining cache and database operations.

use media::WriteCommandsMedia;
use media_admin::WriteCommandsMediaAdmin;
use server_data::db_manager::WriteAccessProvider;

pub mod media;
pub mod media_admin;

pub trait GetWriteCommandsMedia<'a> {
    fn media(self) -> WriteCommandsMedia<'a>;
    fn media_admin(self) -> WriteCommandsMediaAdmin<'a>;
}

impl <'a, I: WriteAccessProvider<'a>> GetWriteCommandsMedia<'a> for I {
    fn media(self) -> WriteCommandsMedia<'a> {
        WriteCommandsMedia::new(self.handle())
    }

    fn media_admin(self) -> WriteCommandsMediaAdmin<'a> {
        WriteCommandsMediaAdmin::new(self.handle())
    }
}
