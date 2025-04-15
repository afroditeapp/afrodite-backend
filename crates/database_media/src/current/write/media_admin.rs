use database::define_current_write_commands;
use model::ProfileContentVersion;

mod media_content;
mod notification;

pub struct InitialModerationRequestIsNowAccepted {
    pub new_profile_content_version: ProfileContentVersion,
}

define_current_write_commands!(CurrentWriteMediaAdmin);

impl<'a> CurrentWriteMediaAdmin<'a> {
    pub fn media_content(self) -> media_content::CurrentWriteMediaAdminMediaContent<'a> {
        media_content::CurrentWriteMediaAdminMediaContent::new(self.cmds)
    }
    pub fn notification(self) -> notification::CurrentWriteMediaAdminNotification<'a> {
        notification::CurrentWriteMediaAdminNotification::new(self.cmds)
    }
}
