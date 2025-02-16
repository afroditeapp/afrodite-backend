use crate::define_cmd_wrapper_read;

mod report;

define_cmd_wrapper_read!(ReadCommandsCommonAdmin);

impl<'a> ReadCommandsCommonAdmin<'a> {
    pub fn report(self) -> report::ReadCommandsCommonAdminReport<'a> {
        report::ReadCommandsCommonAdminReport::new(self.0)
    }
}
