use super::ConnectionProvider;

mod media_content;
mod moderation;

pub struct InitialModerationRequestIsNowAccepted;

define_write_commands!(CurrentWriteMediaAdmin, CurrentSyncWriteMediaAdmin);

impl<C: ConnectionProvider> CurrentSyncWriteMediaAdmin<C> {
    pub fn moderation(self) -> moderation::CurrentSyncWriteMediaAdminModeration<C> {
        moderation::CurrentSyncWriteMediaAdminModeration::new(self.cmds)
    }

    pub fn media_content(self) -> media_content::CurrentSyncWriteMediaAdminMediaContent<C> {
        media_content::CurrentSyncWriteMediaAdminMediaContent::new(self.cmds)
    }
}
