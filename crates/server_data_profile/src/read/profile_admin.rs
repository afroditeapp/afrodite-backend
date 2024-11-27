use server_data::define_cmd_wrapper;

use super::DbReadProfile;

mod profile_name_allowlist;
mod profile_text;

define_cmd_wrapper!(ReadCommandsProfileAdmin);

impl<C: DbReadProfile> ReadCommandsProfileAdmin<C> {
    pub fn profile_name_allowlist(self) -> profile_name_allowlist::ReadCommandsProfileNameAllowlist<C> {
        profile_name_allowlist::ReadCommandsProfileNameAllowlist::new(self.0)
    }

    pub fn profile_text(self) -> profile_text::ReadCommandsProfileText<C> {
        profile_text::ReadCommandsProfileText::new(self.0)
    }
}
