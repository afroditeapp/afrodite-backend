use server_data::define_cmd_wrapper_read;

mod profile_name_allowlist;
mod profile_text;

define_cmd_wrapper_read!(ReadCommandsProfileAdmin);

impl<'a> ReadCommandsProfileAdmin<'a> {
    pub fn profile_name_allowlist(
        self,
    ) -> profile_name_allowlist::ReadCommandsProfileNameAllowlist<'a> {
        profile_name_allowlist::ReadCommandsProfileNameAllowlist::new(self.0)
    }

    pub fn profile_text(self) -> profile_text::ReadCommandsProfileText<'a> {
        profile_text::ReadCommandsProfileText::new(self.0)
    }
}
