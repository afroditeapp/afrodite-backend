use database::define_current_read_commands;

mod data;
mod favorite;
mod profile_name_allowlist;
mod report;
mod notification;

define_current_read_commands!(CurrentReadProfile);

impl<'a> CurrentReadProfile<'a> {
    pub fn data(self) -> data::CurrentReadProfileData<'a> {
        data::CurrentReadProfileData::new(self.cmds)
    }
    pub fn favorite(self) -> favorite::CurrentReadProfileFavorite<'a> {
        favorite::CurrentReadProfileFavorite::new(self.cmds)
    }
    pub fn profile_name_allowlist(
        self,
    ) -> profile_name_allowlist::CurrentReadProfileNameAllowlist<'a> {
        profile_name_allowlist::CurrentReadProfileNameAllowlist::new(self.cmds)
    }
    pub fn report(self) -> report::CurrentReadProfileReport<'a> {
        report::CurrentReadProfileReport::new(self.cmds)
    }
    pub fn notification(self) -> notification::CurrentReadProfileNotification<'a> {
        notification::CurrentReadProfileNotification::new(self.cmds)
    }
}
