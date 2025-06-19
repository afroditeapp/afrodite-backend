use crate::define_current_read_commands;

mod notification;
mod report;
mod statistics;

define_current_read_commands!(CurrentReadCommonAdmin);

impl<'a> CurrentReadCommonAdmin<'a> {
    pub fn notification(self) -> notification::CurrentReadAccountAdminNotification<'a> {
        notification::CurrentReadAccountAdminNotification::new(self.cmds)
    }
    pub fn statistics(self) -> statistics::CurrentReadAccountAdminStatistics<'a> {
        statistics::CurrentReadAccountAdminStatistics::new(self.cmds)
    }
    pub fn report(self) -> report::CurrentReadCommonAdminReport<'a> {
        report::CurrentReadCommonAdminReport::new(self.cmds)
    }
}
