
use media::ReadCommandsMedia;
use media_admin::ReadCommandsMediaAdmin;
use server_data::read::{ReadCommands, ReadCommandsProvider};

pub mod media;
pub mod media_admin;

pub trait GetReadMediaCommands<C: ReadCommandsProvider> {
    fn media(self) -> ReadCommandsMedia<C>;
    fn media_admin(self) -> ReadCommandsMediaAdmin<C>;
}

impl <C: ReadCommandsProvider> GetReadMediaCommands<C> for C {
    fn media(self) -> ReadCommandsMedia<C> {
        ReadCommandsMedia::new(self)
    }

    fn media_admin(self) -> ReadCommandsMediaAdmin<C> {
        ReadCommandsMediaAdmin::new(self)
    }
}
