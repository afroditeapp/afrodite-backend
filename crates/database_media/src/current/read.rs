use database::DbReadAccessProvider;

use self::{media::CurrentReadMedia, media_admin::CurrentReadMediaAdmin};

pub mod media;
pub mod media_admin;

pub trait GetDbReadCommandsMedia {
    fn media(&mut self) -> CurrentReadMedia<'_>;
    fn media_admin(&mut self) -> CurrentReadMediaAdmin<'_>;
}

impl<I: DbReadAccessProvider> GetDbReadCommandsMedia for I {
    fn media(&mut self) -> CurrentReadMedia<'_> {
        CurrentReadMedia::new(self.handle())
    }
    fn media_admin(&mut self) -> CurrentReadMediaAdmin<'_> {
        CurrentReadMediaAdmin::new(self.handle())
    }
}
