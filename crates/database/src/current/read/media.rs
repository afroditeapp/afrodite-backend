




use simple_backend_database::diesel_db::{ConnectionProvider};



mod media_content;
mod moderation_request;

define_read_commands!(CurrentReadMedia, CurrentSyncReadMedia);

impl<C: ConnectionProvider> CurrentSyncReadMedia<C> {
    pub fn media_content(self) -> media_content::CurrentSyncReadMediaMediaContent<C> {
        media_content::CurrentSyncReadMediaMediaContent::new(self.cmds)
    }

    pub fn moderation_request(self) -> moderation_request::CurrentSyncReadMediaModerationRequest<C> {
        moderation_request::CurrentSyncReadMediaModerationRequest::new(self.cmds)
    }
}
