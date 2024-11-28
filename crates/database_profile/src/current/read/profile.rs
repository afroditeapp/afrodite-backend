use database::define_current_read_commands;

mod data;
mod favorite;
mod profile_name_allowlist;

define_current_read_commands!(CurrentReadProfile);

impl<'a> CurrentReadProfile<'a> {
    pub fn data(self) -> data::CurrentReadProfileData<'a> {
        data::CurrentReadProfileData::new(self.cmds)
    }
    pub fn favorite(self) -> favorite::CurrentReadProfileFavorite<'a> {
        favorite::CurrentReadProfileFavorite::new(self.cmds)
    }
    pub fn profile_name_allowlist(self) -> profile_name_allowlist::CurrentReadProfileNameAllowlist<'a> {
        profile_name_allowlist::CurrentReadProfileNameAllowlist::new(self.cmds)
    }
}
