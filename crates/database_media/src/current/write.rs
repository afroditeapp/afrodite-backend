use database::DbWriteAccessProvider;

use self::{media::CurrentWriteMedia, media_admin::CurrentWriteMediaAdmin};

pub mod media;
pub mod media_admin;

pub trait GetDbWriteCommandsMedia {
    fn media(&mut self) -> CurrentWriteMedia;
    fn media_admin(&mut self) -> CurrentWriteMediaAdmin;
}

impl <I: DbWriteAccessProvider> GetDbWriteCommandsMedia for I {
    fn media(&mut self) -> CurrentWriteMedia {
        CurrentWriteMedia::new(self.handle())
    }
    fn media_admin(&mut self) -> CurrentWriteMediaAdmin {
        CurrentWriteMediaAdmin::new(self.handle())
    }
}
