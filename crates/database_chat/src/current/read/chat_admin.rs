use database::define_current_read_commands;

mod report;

define_current_read_commands!(CurrentReadChatAdmin);

impl<'a> CurrentReadChatAdmin<'a> {
    pub fn report(self) -> report::CurrentReadChatAdminReport<'a> {
        report::CurrentReadChatAdminReport::new(self.cmds)
    }
}
