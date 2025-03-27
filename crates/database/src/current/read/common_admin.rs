use crate::define_current_read_commands;

mod api_usage;
mod report;

define_current_read_commands!(CurrentReadCommonAdmin);

impl<'a> CurrentReadCommonAdmin<'a> {
    pub fn api_usage(self) -> api_usage::CurrentReadAccountAdminApiUsage<'a> {
        api_usage::CurrentReadAccountAdminApiUsage::new(self.cmds)
    }
    pub fn report(self) -> report::CurrentReadCommonAdminReport<'a> {
        report::CurrentReadCommonAdminReport::new(self.cmds)
    }
}
