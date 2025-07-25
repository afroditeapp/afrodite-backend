use database::define_current_read_commands;

mod data;
mod favorite;
mod moderation;
mod notification;
mod report;

define_current_read_commands!(CurrentReadProfile);

impl<'a> CurrentReadProfile<'a> {
    pub fn data(self) -> data::CurrentReadProfileData<'a> {
        data::CurrentReadProfileData::new(self.cmds)
    }
    pub fn favorite(self) -> favorite::CurrentReadProfileFavorite<'a> {
        favorite::CurrentReadProfileFavorite::new(self.cmds)
    }
    pub fn report(self) -> report::CurrentReadProfileReport<'a> {
        report::CurrentReadProfileReport::new(self.cmds)
    }
    pub fn notification(self) -> notification::CurrentReadProfileNotification<'a> {
        notification::CurrentReadProfileNotification::new(self.cmds)
    }
    pub fn moderation(self) -> moderation::CurrentReadProfileModeration<'a> {
        moderation::CurrentReadProfileModeration::new(self.cmds)
    }
}
