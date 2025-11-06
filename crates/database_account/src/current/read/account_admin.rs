use database::define_current_read_commands;

define_current_read_commands!(CurrentReadAccountAdmin);

mod login;
mod news;

impl<'a> CurrentReadAccountAdmin<'a> {
    pub fn login(self) -> login::CurrentReadAccountLock<'a> {
        login::CurrentReadAccountLock::new(self.cmds)
    }

    pub fn news(self) -> news::CurrentReadAccountNewsAdmin<'a> {
        news::CurrentReadAccountNewsAdmin::new(self.cmds)
    }
}
