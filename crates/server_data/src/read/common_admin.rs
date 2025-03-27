use crate::define_cmd_wrapper_read;

mod api_usage;
mod report;

define_cmd_wrapper_read!(ReadCommandsCommonAdmin);

impl<'a> ReadCommandsCommonAdmin<'a> {
    pub fn report(self) -> report::ReadCommandsCommonAdminReport<'a> {
        report::ReadCommandsCommonAdminReport::new(self.0)
    }
    pub fn api_usage(self) -> api_usage::ReadCommandsCommonAdminApiUsage<'a> {
        api_usage::ReadCommandsCommonAdminApiUsage::new(self.0)
    }
}
