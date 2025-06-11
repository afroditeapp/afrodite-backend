use crate::define_current_write_commands;

mod notification;
mod statistics;
mod report;

define_current_write_commands!(CurrentWriteCommonAdmin);

impl<'a> CurrentWriteCommonAdmin<'a> {
    pub fn notification(self) -> notification::CurrentWriteCommonAdminNotification<'a> {
        notification::CurrentWriteCommonAdminNotification::new(self.cmds)
    }
    pub fn report(self) -> report::CurrentWriteCommonAdminReport<'a> {
        report::CurrentWriteCommonAdminReport::new(self.cmds)
    }
    pub fn statistics(self) -> statistics::CurrentWriteCommonStatistics<'a> {
        statistics::CurrentWriteCommonStatistics::new(self.cmds)
    }
}
