use crate::define_current_write_commands;

mod statistics;
mod report;

define_current_write_commands!(CurrentWriteCommonAdmin);

impl<'a> CurrentWriteCommonAdmin<'a> {
    pub fn report(self) -> report::CurrentWriteCommonAdminReport<'a> {
        report::CurrentWriteCommonAdminReport::new(self.cmds)
    }
    pub fn statistics(self) -> statistics::CurrentWriteCommonStatistics<'a> {
        statistics::CurrentWriteCommonStatistics::new(self.cmds)
    }
}
