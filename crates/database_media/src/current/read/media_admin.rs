use database::define_current_read_commands;
use database::ConnectionProvider;

mod moderation;
mod moderation_request;

define_current_read_commands!(CurrentReadMediaAdmin, CurrentSyncReadMediaAdmin);

impl<C: ConnectionProvider> CurrentSyncReadMediaAdmin<C> {
    pub fn moderation_request(
        self,
    ) -> moderation_request::CurrentSyncReadMediaAdminModerationRequest<C> {
        moderation_request::CurrentSyncReadMediaAdminModerationRequest::new(self.cmds)
    }

    pub fn moderation(self) -> moderation::CurrentSyncReadMediaAdminModeration<C> {
        moderation::CurrentSyncReadMediaAdminModeration::new(self.cmds)
    }
}
