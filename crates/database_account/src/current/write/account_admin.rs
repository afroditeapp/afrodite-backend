use database::define_current_write_commands;

mod news;

define_current_write_commands!(CurrentWriteAccountAdmin);

impl <'a> CurrentWriteAccountAdmin<'a> {
    pub fn news(self) -> news::CurrentWriteAccountNewsAdmin<'a> {
        news::CurrentWriteAccountNewsAdmin::new(self.cmds)
    }
}
