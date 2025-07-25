use server_data::define_cmd_wrapper_read;

mod iterator;
mod moderation;

define_cmd_wrapper_read!(ReadCommandsProfileAdmin);

impl<'a> ReadCommandsProfileAdmin<'a> {
    pub fn moderation(self) -> moderation::ReadCommandsProfileModeration<'a> {
        moderation::ReadCommandsProfileModeration::new(self.0)
    }

    pub fn iterator(self) -> iterator::ReadCommandsProfileIterator<'a> {
        iterator::ReadCommandsProfileIterator::new(self.0)
    }
}
