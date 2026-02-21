use server_data::define_cmd_wrapper_read;

mod iterator;
mod moderation;
pub mod profile_attributes;

define_cmd_wrapper_read!(ReadCommandsProfileAdmin);

impl<'a> ReadCommandsProfileAdmin<'a> {
    pub fn attribute_schema(
        self,
    ) -> profile_attributes::ReadCommandsProfileAdminAttributeSchema<'a> {
        profile_attributes::ReadCommandsProfileAdminAttributeSchema::new(self.0)
    }

    pub fn moderation(self) -> moderation::ReadCommandsProfileModeration<'a> {
        moderation::ReadCommandsProfileModeration::new(self.0)
    }

    pub fn iterator(self) -> iterator::ReadCommandsProfileIterator<'a> {
        iterator::ReadCommandsProfileIterator::new(self.0)
    }
}
