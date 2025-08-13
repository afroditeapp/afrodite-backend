use database::define_current_write_commands;

mod media_content;
mod notification;

define_current_write_commands!(CurrentWriteMediaAdmin);

impl<'a> CurrentWriteMediaAdmin<'a> {
    pub fn media_content(self) -> media_content::CurrentWriteMediaAdminMediaContent<'a> {
        media_content::CurrentWriteMediaAdminMediaContent::new(self.cmds)
    }
    pub fn notification(self) -> notification::CurrentWriteMediaAdminNotification<'a> {
        notification::CurrentWriteMediaAdminNotification::new(self.cmds)
    }
}
