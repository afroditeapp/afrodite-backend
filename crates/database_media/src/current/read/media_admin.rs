use database::define_current_read_commands;

mod content;
mod report;

define_current_read_commands!(CurrentReadMediaAdmin);

impl<'a> CurrentReadMediaAdmin<'a> {
    pub fn content(self) -> content::CurrentReadMediaAdminContent<'a> {
        content::CurrentReadMediaAdminContent::new(self.cmds)
    }
    pub fn report(self) -> report::CurrentReadMediaAdminReport<'a> {
        report::CurrentReadMediaAdminReport::new(self.cmds)
    }
}
