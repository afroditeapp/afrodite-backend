use database::define_current_write_commands;

mod profile_name_allowlist;
mod profile_text;

define_current_write_commands!(CurrentWriteProfileAdmin);

impl <'a> CurrentWriteProfileAdmin<'a> {
    pub fn profile_name_allowlist(self) -> profile_name_allowlist::CurrentWriteProfileAdminProfileNameAllowlist<'a> {
        profile_name_allowlist::CurrentWriteProfileAdminProfileNameAllowlist::new(self.cmds)
    }
    pub fn profile_text(self) -> profile_text::CurrentWriteProfileAdminProfileText<'a> {
        profile_text::CurrentWriteProfileAdminProfileText::new(self.cmds)
    }
}
