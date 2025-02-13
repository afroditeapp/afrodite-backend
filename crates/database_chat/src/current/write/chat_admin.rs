use database::define_current_write_commands;

mod report;

define_current_write_commands!(CurrentWriteChatAdmin);

impl<'a> CurrentWriteChatAdmin<'a> {
    pub fn report(self) -> report::CurrentWriteChatAdminReport<'a> {
        report::CurrentWriteChatAdminReport::new(self.cmds)
    }
}
