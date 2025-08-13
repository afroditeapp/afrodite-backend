use database::define_current_read_commands;

mod iterator;
mod moderation;

define_current_read_commands!(CurrentReadProfileAdmin);

impl<'a> CurrentReadProfileAdmin<'a> {
    pub fn moderation(self) -> moderation::CurrentReadProfileModeration<'a> {
        moderation::CurrentReadProfileModeration::new(self.cmds)
    }

    pub fn iterator(self) -> iterator::CurrentReadProfileIterator<'a> {
        iterator::CurrentReadProfileIterator::new(self.cmds)
    }
}
