use database::DbWriteAccessProvider;

use self::{media::CurrentWriteMedia, media_admin::CurrentWriteMediaAdmin};

pub mod media;
pub mod media_admin;

pub trait GetDbWriteCommandsMedia {
    fn media(&mut self) -> CurrentWriteMedia<'_>;
    fn media_admin(&mut self) -> CurrentWriteMediaAdmin<'_>;
}

impl<I: DbWriteAccessProvider> GetDbWriteCommandsMedia for I {
    fn media(&mut self) -> CurrentWriteMedia<'_> {
        CurrentWriteMedia::new(self.handle())
    }
    fn media_admin(&mut self) -> CurrentWriteMediaAdmin<'_> {
        CurrentWriteMediaAdmin::new(self.handle())
    }
}
