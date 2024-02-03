use super::ConnectionProvider;

mod media_content;
mod moderation_request;

define_write_commands!(CurrentWriteMedia, CurrentSyncWriteMedia);

pub struct DeletedSomething;

impl<C: ConnectionProvider> CurrentSyncWriteMedia<C> {
    pub fn media_content(self) -> media_content::CurrentSyncWriteMediaContent<C> {
        media_content::CurrentSyncWriteMediaContent::new(self.cmds)
    }

    pub fn moderation_request(
        self,
    ) -> moderation_request::CurrentSyncWriteMediaModerationRequest<C> {
        moderation_request::CurrentSyncWriteMediaModerationRequest::new(self.cmds)
    }
}
