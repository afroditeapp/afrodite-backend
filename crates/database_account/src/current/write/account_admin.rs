use database::define_current_write_commands;

mod ban;
mod login;
mod news;

define_current_write_commands!(CurrentWriteAccountAdmin);

impl<'a> CurrentWriteAccountAdmin<'a> {
    pub fn ban(self) -> ban::CurrentWriteAccountBanAdmin<'a> {
        ban::CurrentWriteAccountBanAdmin::new(self.cmds)
    }

    pub fn login(self) -> login::CurrentWriteAccountLockAdmin<'a> {
        login::CurrentWriteAccountLockAdmin::new(self.cmds)
    }

    pub fn news(self) -> news::CurrentWriteAccountNewsAdmin<'a> {
        news::CurrentWriteAccountNewsAdmin::new(self.cmds)
    }
}
