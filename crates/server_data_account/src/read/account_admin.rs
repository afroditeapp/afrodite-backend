use server_data::define_cmd_wrapper_read;

pub mod news;
pub mod permissions;
pub mod search;

define_cmd_wrapper_read!(ReadCommandsAccountAdmin);

impl<'a> ReadCommandsAccountAdmin<'a> {
    pub fn news(self) -> news::ReadCommandsAccountNewsAdmin<'a> {
        news::ReadCommandsAccountNewsAdmin::new(self.0)
    }
    pub fn search(self) -> search::ReadCommandsAccountSearchAdmin<'a> {
        search::ReadCommandsAccountSearchAdmin::new(self.0)
    }
    pub fn permissions(self) -> permissions::ReadCommandsAccountPermissionsAdmin<'a> {
        permissions::ReadCommandsAccountPermissionsAdmin::new(self.0)
    }
}
