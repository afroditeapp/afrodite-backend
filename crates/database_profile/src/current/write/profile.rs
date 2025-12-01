use database::define_current_write_commands;

mod data;
mod favorite;
mod moderation;
mod notification;
mod privacy;
mod report;
mod search;

define_current_write_commands!(CurrentWriteProfile);

impl<'a> CurrentWriteProfile<'a> {
    pub fn data(self) -> data::CurrentWriteProfileData<'a> {
        data::CurrentWriteProfileData::new(self.cmds)
    }

    pub fn favorite(self) -> favorite::CurrentWriteProfileFavorite<'a> {
        favorite::CurrentWriteProfileFavorite::new(self.cmds)
    }

    pub fn moderation(self) -> moderation::CurrentWriteModeration<'a> {
        moderation::CurrentWriteModeration::new(self.cmds)
    }

    pub fn report(self) -> report::CurrentWriteProfileReport<'a> {
        report::CurrentWriteProfileReport::new(self.cmds)
    }

    pub fn notification(self) -> notification::CurrentWriteProfileNotification<'a> {
        notification::CurrentWriteProfileNotification::new(self.cmds)
    }

    pub fn privacy(self) -> privacy::CurrentWriteProfilePrivacy<'a> {
        privacy::CurrentWriteProfilePrivacy::new(self.cmds)
    }

    pub fn search(self) -> search::CurrentWriteProfileSearch<'a> {
        search::CurrentWriteProfileSearch::new(self.cmds)
    }
}
