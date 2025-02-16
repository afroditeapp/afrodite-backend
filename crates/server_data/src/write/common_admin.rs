use crate::define_cmd_wrapper_write;

mod report;

define_cmd_wrapper_write!(WriteCommandsCommonAdmin);

impl<'a> WriteCommandsCommonAdmin<'a> {
    pub fn report(self) -> report::WriteCommandsCommonAdminReport<'a> {
        report::WriteCommandsCommonAdminReport::new(self.0)
    }
}
