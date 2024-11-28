use database::define_current_read_commands;

mod moderation;
mod moderation_request;

define_current_read_commands!(CurrentReadMediaAdmin);

impl<'a> CurrentReadMediaAdmin<'a> {
    pub fn moderation_request(
        self,
    ) -> moderation_request::CurrentReadMediaAdminModerationRequest<'a> {
        moderation_request::CurrentReadMediaAdminModerationRequest::new(self.cmds)
    }

    pub fn moderation(self) -> moderation::CurrentReadMediaAdminModeration<'a> {
        moderation::CurrentReadMediaAdminModeration::new(self.cmds)
    }
}
