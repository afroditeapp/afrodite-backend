use database::define_current_read_commands;

define_current_read_commands!(CurrentReadAccountAdmin);

mod news;
mod search;

impl<'a> CurrentReadAccountAdmin<'a> {
    pub fn news(self) -> news::CurrentReadAccountNewsAdmin<'a> {
        news::CurrentReadAccountNewsAdmin::new(self.cmds)
    }
    pub fn search(self) -> search::CurrentReadAccountSearchAdmin<'a> {
        search::CurrentReadAccountSearchAdmin::new(self.cmds)
    }
}
