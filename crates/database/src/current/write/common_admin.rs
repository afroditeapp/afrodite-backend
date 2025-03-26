use crate::define_current_write_commands;

mod api_usage;
mod report;

define_current_write_commands!(CurrentWriteCommonAdmin);

impl<'a> CurrentWriteCommonAdmin<'a> {
    pub fn report(self) -> report::CurrentWriteCommonAdminReport<'a> {
        report::CurrentWriteCommonAdminReport::new(self.cmds)
    }
    pub fn api_usage(self) -> api_usage::CurrentWriteCommonApiUsage<'a> {
        api_usage::CurrentWriteCommonApiUsage::new(self.cmds)
    }
}
