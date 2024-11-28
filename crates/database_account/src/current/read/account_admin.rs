use database::define_current_read_commands;

define_current_read_commands!(CurrentReadAccountAdmin);

mod news;

impl<'a> CurrentReadAccountAdmin<'a> {
    pub fn news(self) -> news::CurrentReadAccountNewsAdmin<'a> {
        news::CurrentReadAccountNewsAdmin::new(self.cmds)
    }
}
