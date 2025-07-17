use server_data::define_cmd_wrapper_write;

pub mod content;
mod notification;

define_cmd_wrapper_write!(WriteCommandsMediaAdmin);

impl<'a> WriteCommandsMediaAdmin<'a> {
    pub fn content(self) -> content::WriteCommandsProfileAdminContent<'a> {
        content::WriteCommandsProfileAdminContent::new(self.0)
    }
    pub fn notification(self) -> notification::WriteCommandsMediaAdminNotification<'a> {
        notification::WriteCommandsMediaAdminNotification::new(self.0)
    }
}
