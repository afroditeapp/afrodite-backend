use server_data::define_cmd_wrapper_write;

pub mod notification;
pub mod profile_name_allowlist;
pub mod profile_text;

define_cmd_wrapper_write!(WriteCommandsProfileAdmin);

impl<'a> WriteCommandsProfileAdmin<'a> {
    pub fn profile_name_allowlist(
        self,
    ) -> profile_name_allowlist::WriteCommandsProfileAdminProfileNameAllowlist<'a> {
        profile_name_allowlist::WriteCommandsProfileAdminProfileNameAllowlist::new(self.0)
    }

    pub fn profile_text(self) -> profile_text::WriteCommandsProfileAdminProfileText<'a> {
        profile_text::WriteCommandsProfileAdminProfileText::new(self.0)
    }

    pub fn notification(self) -> notification::WriteCommandsProfileAdminNotification<'a> {
        notification::WriteCommandsProfileAdminNotification::new(self.0)
    }
}
