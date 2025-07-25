use server_data::define_cmd_wrapper_write;

pub mod moderation;
pub mod notification;

define_cmd_wrapper_write!(WriteCommandsProfileAdmin);

impl<'a> WriteCommandsProfileAdmin<'a> {
    pub fn moderation(self) -> moderation::WriteCommandsProfileAdminModeration<'a> {
        moderation::WriteCommandsProfileAdminModeration::new(self.0)
    }

    pub fn notification(self) -> notification::WriteCommandsProfileAdminNotification<'a> {
        notification::WriteCommandsProfileAdminNotification::new(self.0)
    }
}
