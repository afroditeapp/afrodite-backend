use server_data::define_cmd_wrapper;

pub mod profile_name_allowlist;
pub mod profile_text;

define_cmd_wrapper!(WriteCommandsProfileAdmin);

impl<C> WriteCommandsProfileAdmin<C> {
    pub fn profile_name_allowlist(self) -> profile_name_allowlist::WriteCommandsProfileAdminProfileNameAllowlist<C> {
        profile_name_allowlist::WriteCommandsProfileAdminProfileNameAllowlist::new(self.0)
    }

    pub fn profile_text(self) -> profile_text::WriteCommandsProfileAdminProfileText<C> {
        profile_text::WriteCommandsProfileAdminProfileText::new(self.0)
    }
}
