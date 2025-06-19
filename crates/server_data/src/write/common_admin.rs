use crate::define_cmd_wrapper_write;

mod notification;
mod report;
mod statistics;

define_cmd_wrapper_write!(WriteCommandsCommonAdmin);

impl<'a> WriteCommandsCommonAdmin<'a> {
    pub fn notification(self) -> notification::WriteCommandsCommonAdminNotification<'a> {
        notification::WriteCommandsCommonAdminNotification::new(self.0)
    }

    pub fn report(self) -> report::WriteCommandsCommonAdminReport<'a> {
        report::WriteCommandsCommonAdminReport::new(self.0)
    }

    pub fn statistics(self) -> statistics::WriteCommandsCommonAdminStatistics<'a> {
        statistics::WriteCommandsCommonAdminStatistics::new(self.0)
    }
}
