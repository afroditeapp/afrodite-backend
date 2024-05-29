
use media::ReadCommandsMedia;
use media_admin::ReadCommandsMediaAdmin;
use server_data::{read::ReadCommands};

pub mod media;
pub mod media_admin;

pub trait GetReadMediaCommands<'a>: Sized {
    fn media(self) -> ReadCommandsMedia<'a>;
    fn media_admin(self) -> ReadCommandsMediaAdmin<'a>;
}

impl <'a> GetReadMediaCommands<'a> for ReadCommands<'a> {
    fn media(self) -> ReadCommandsMedia<'a> {
        ReadCommandsMedia::new(self)
    }

    fn media_admin(self) -> ReadCommandsMediaAdmin<'a> {
        ReadCommandsMediaAdmin::new(self)
    }
}
