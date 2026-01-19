use database::define_current_write_commands;

mod image_processing_config;
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
    pub fn image_processing_config(
        self,
    ) -> image_processing_config::CurrentWriteMediaAdminImageProcessingConfig<'a> {
        image_processing_config::CurrentWriteMediaAdminImageProcessingConfig::new(self.cmds)
    }
}
