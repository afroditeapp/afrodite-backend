use crate::define_current_read_commands;

mod report;

define_current_read_commands!(CurrentReadCommonAdmin);

impl<'a> CurrentReadCommonAdmin<'a> {
    pub fn report(self) -> report::CurrentReadCommonAdminReport<'a> {
        report::CurrentReadCommonAdminReport::new(self.cmds)
    }
}
