use database::define_history_read_commands;

use self::{media::HistoryReadMedia, media_admin::HistoryReadMediaAdmin};

pub mod media;
pub mod media_admin;

define_history_read_commands!(HistorySyncReadCommands);

impl<'a> HistorySyncReadCommands<'a> {
    pub fn into_media(self) -> HistoryReadMedia<'a> {
        HistoryReadMedia::new(self.cmds)
    }

    pub fn into_media_admin(self) -> HistoryReadMediaAdmin<'a> {
        HistoryReadMediaAdmin::new(self.cmds)
    }
}
