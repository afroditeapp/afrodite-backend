use crate::define_cmd_wrapper_read;

mod notification;
mod report;
mod statistics;

define_cmd_wrapper_read!(ReadCommandsCommonAdmin);

impl<'a> ReadCommandsCommonAdmin<'a> {
    pub fn notification(self) -> notification::ReadCommandsCommonAdminNotification<'a> {
        notification::ReadCommandsCommonAdminNotification::new(self.0)
    }
    pub fn report(self) -> report::ReadCommandsCommonAdminReport<'a> {
        report::ReadCommandsCommonAdminReport::new(self.0)
    }
    pub fn statistics(self) -> statistics::ReadCommandsCommonAdminStatistics<'a> {
        statistics::ReadCommandsCommonAdminStatistics::new(self.0)
    }
}
