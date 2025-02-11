use database::define_current_read_commands;

mod profile_name_allowlist;
mod profile_text;
mod iterator;
mod report;

define_current_read_commands!(CurrentReadProfileAdmin);

impl<'a> CurrentReadProfileAdmin<'a> {
    pub fn profile_name_allowlist(
        self,
    ) -> profile_name_allowlist::CurrentReadProfileNameAllowlist<'a> {
        profile_name_allowlist::CurrentReadProfileNameAllowlist::new(self.cmds)
    }

    pub fn profile_text(self) -> profile_text::CurrentReadProfileText<'a> {
        profile_text::CurrentReadProfileText::new(self.cmds)
    }

    pub fn iterator(self) -> iterator::CurrentReadProfileIterator<'a> {
        iterator::CurrentReadProfileIterator::new(self.cmds)
    }

    pub fn report(self) -> report::CurrentReadProfileReport<'a> {
        report::CurrentReadProfileReport::new(self.cmds)
    }
}
