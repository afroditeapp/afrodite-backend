//! Synchronous write commands combining cache and database operations.

use media::WriteCommandsMedia;
use media_admin::WriteCommandsMediaAdmin;
use server_data::db_manager::WriteAccessProvider;

pub mod media;
pub mod media_admin;

pub trait GetWriteCommandsMedia {
    fn media(&self) -> WriteCommandsMedia<'_>;
    fn media_admin(&self) -> WriteCommandsMediaAdmin<'_>;
}

impl<I: WriteAccessProvider> GetWriteCommandsMedia for I {
    fn media(&self) -> WriteCommandsMedia<'_> {
        WriteCommandsMedia::new(self.handle())
    }

    fn media_admin(&self) -> WriteCommandsMediaAdmin<'_> {
        WriteCommandsMediaAdmin::new(self.handle())
    }
}
