use crate::define_cmd_wrapper_write;

mod api_usage;
mod report;

define_cmd_wrapper_write!(WriteCommandsCommonAdmin);

impl<'a> WriteCommandsCommonAdmin<'a> {
    pub fn report(self) -> report::WriteCommandsCommonAdminReport<'a> {
        report::WriteCommandsCommonAdminReport::new(self.0)
    }

    pub fn api_usage(self) -> api_usage::WriteCommandsCommonAdminApiUsage<'a> {
        api_usage::WriteCommandsCommonAdminApiUsage::new(self.0)
    }
}
