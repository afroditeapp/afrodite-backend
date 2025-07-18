use server_data::define_cmd_wrapper_read;

mod iterator;
mod profile_name;
mod profile_text;

define_cmd_wrapper_read!(ReadCommandsProfileAdmin);

impl<'a> ReadCommandsProfileAdmin<'a> {
    pub fn profile_name_allowlist(self) -> profile_name::ReadCommandsProfileName<'a> {
        profile_name::ReadCommandsProfileName::new(self.0)
    }

    pub fn profile_text(self) -> profile_text::ReadCommandsProfileText<'a> {
        profile_text::ReadCommandsProfileText::new(self.0)
    }

    pub fn iterator(self) -> iterator::ReadCommandsProfileIterator<'a> {
        iterator::ReadCommandsProfileIterator::new(self.0)
    }
}
