use database::define_current_read_commands;

mod iterator;
mod profile_name;
mod profile_text;
mod search;

define_current_read_commands!(CurrentReadProfileAdmin);

impl<'a> CurrentReadProfileAdmin<'a> {
    pub fn profile_name(self) -> profile_name::CurrentReadProfileName<'a> {
        profile_name::CurrentReadProfileName::new(self.cmds)
    }

    pub fn profile_text(self) -> profile_text::CurrentReadProfileText<'a> {
        profile_text::CurrentReadProfileText::new(self.cmds)
    }

    pub fn iterator(self) -> iterator::CurrentReadProfileIterator<'a> {
        iterator::CurrentReadProfileIterator::new(self.cmds)
    }

    pub fn search(self) -> search::CurrentReadProfileAdminSearch<'a> {
        search::CurrentReadProfileAdminSearch::new(self.cmds)
    }
}
