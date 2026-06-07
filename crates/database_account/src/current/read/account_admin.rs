use database::define_current_read_commands;

define_current_read_commands!(CurrentReadAccountAdmin);

mod custom_email;
mod login;
mod news;

impl<'a> CurrentReadAccountAdmin<'a> {
    pub fn login(self) -> login::CurrentReadAccountLock<'a> {
        login::CurrentReadAccountLock::new(self.cmds)
    }

    pub fn news(self) -> news::CurrentReadAccountNewsAdmin<'a> {
        news::CurrentReadAccountNewsAdmin::new(self.cmds)
    }

    pub fn custom_email(self) -> custom_email::CurrentReadAccountCustomEmailAdmin<'a> {
        custom_email::CurrentReadAccountCustomEmailAdmin::new(self.cmds)
    }
}
