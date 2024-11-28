use database::define_current_write_commands;
use model::ProfileContentVersion;

mod media_content;
mod moderation;

pub struct InitialModerationRequestIsNowAccepted {
    pub new_profile_content_version: ProfileContentVersion,
}

define_current_write_commands!(CurrentWriteMediaAdmin);

impl <'a> CurrentWriteMediaAdmin<'a> {
    pub fn moderation(self) -> moderation::CurrentWriteMediaAdminModeration<'a> {
        moderation::CurrentWriteMediaAdminModeration::new(self.cmds)
    }

    pub fn media_content(self) -> media_content::CurrentWriteMediaAdminMediaContent<'a> {
        media_content::CurrentWriteMediaAdminMediaContent::new(self.cmds)
    }
}
