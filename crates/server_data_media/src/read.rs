use media::ReadCommandsMedia;
use media_admin::ReadCommandsMediaAdmin;
use server_data::db_manager::ReadAccessProvider;

pub mod media;
pub mod media_admin;

pub trait GetReadMediaCommands<'a> {
    fn media(self) -> ReadCommandsMedia<'a>;
    fn media_admin(self) -> ReadCommandsMediaAdmin<'a>;
}

impl<'a, I: ReadAccessProvider<'a>> GetReadMediaCommands<'a> for I {
    fn media(self) -> ReadCommandsMedia<'a> {
        ReadCommandsMedia::new(self.handle())
    }

    fn media_admin(self) -> ReadCommandsMediaAdmin<'a> {
        ReadCommandsMediaAdmin::new(self.handle())
    }
}
