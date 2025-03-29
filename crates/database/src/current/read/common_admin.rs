use crate::define_current_read_commands;

mod statistics;
mod report;

define_current_read_commands!(CurrentReadCommonAdmin);

impl<'a> CurrentReadCommonAdmin<'a> {
    pub fn statistics(self) -> statistics::CurrentReadAccountAdminStatistics<'a> {
        statistics::CurrentReadAccountAdminStatistics::new(self.cmds)
    }
    pub fn report(self) -> report::CurrentReadCommonAdminReport<'a> {
        report::CurrentReadCommonAdminReport::new(self.cmds)
    }
}
