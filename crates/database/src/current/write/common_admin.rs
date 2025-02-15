use crate::define_current_write_commands;

mod report;

define_current_write_commands!(CurrentWriteCommonAdmin);

impl<'a> CurrentWriteCommonAdmin<'a> {
    pub fn report(self) -> report::CurrentWriteCommonAdminReport<'a> {
        report::CurrentWriteCommonAdminReport::new(self.cmds)
    }
}
