use database::define_current_read_commands;

mod content;
mod image_processing_config;

define_current_read_commands!(CurrentReadMediaAdmin);

impl<'a> CurrentReadMediaAdmin<'a> {
    pub fn content(self) -> content::CurrentReadMediaAdminContent<'a> {
        content::CurrentReadMediaAdminContent::new(self.cmds)
    }

    pub fn image_processing_config(
        self,
    ) -> image_processing_config::CurrentReadMediaAdminImageProcessingConfig<'a> {
        image_processing_config::CurrentReadMediaAdminImageProcessingConfig::new(self.cmds)
    }
}
