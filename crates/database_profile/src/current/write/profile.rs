use database::define_current_write_commands;

mod data;
mod favorite;
mod profile_name_allowlist;
mod profile_text;
mod report;
mod notification;

define_current_write_commands!(CurrentWriteProfile);

impl<'a> CurrentWriteProfile<'a> {
    pub fn data(self) -> data::CurrentWriteProfileData<'a> {
        data::CurrentWriteProfileData::new(self.cmds)
    }

    pub fn favorite(self) -> favorite::CurrentWriteProfileFavorite<'a> {
        favorite::CurrentWriteProfileFavorite::new(self.cmds)
    }

    pub fn profile_name_allowlist(
        self,
    ) -> profile_name_allowlist::CurrentWriteProfileNameAllowlist<'a> {
        profile_name_allowlist::CurrentWriteProfileNameAllowlist::new(self.cmds)
    }

    pub fn profile_text(self) -> profile_text::CurrentWriteProfileText<'a> {
        profile_text::CurrentWriteProfileText::new(self.cmds)
    }

    pub fn report(self) -> report::CurrentWriteProfileReport<'a> {
        report::CurrentWriteProfileReport::new(self.cmds)
    }

    pub fn notification(self) -> notification::CurrentWriteProfileNotification<'a> {
        notification::CurrentWriteProfileNotification::new(self.cmds)
    }
}
