use server_data::define_cmd_wrapper_read;

pub mod login;
pub mod news;
pub mod permissions;
define_cmd_wrapper_read!(ReadCommandsAccountAdmin);

impl<'a> ReadCommandsAccountAdmin<'a> {
    pub fn login(self) -> login::ReadCommandsAccountLockAdmin<'a> {
        login::ReadCommandsAccountLockAdmin::new(self.0)
    }

    pub fn news(self) -> news::ReadCommandsAccountNewsAdmin<'a> {
        news::ReadCommandsAccountNewsAdmin::new(self.0)
    }
    pub fn permissions(self) -> permissions::ReadCommandsAccountPermissionsAdmin<'a> {
        permissions::ReadCommandsAccountPermissionsAdmin::new(self.0)
    }
}
