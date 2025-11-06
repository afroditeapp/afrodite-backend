use server_data::define_cmd_wrapper_write;

mod ban;
mod login;
mod news;
mod permissions;
define_cmd_wrapper_write!(WriteCommandsAccountAdmin);

impl<'a> WriteCommandsAccountAdmin<'a> {
    pub fn ban(self) -> ban::WriteCommandsAccountBan<'a> {
        ban::WriteCommandsAccountBan::new(self.0)
    }

    pub fn login(self) -> login::WriteCommandsAccountLockAdmin<'a> {
        login::WriteCommandsAccountLockAdmin::new(self.0)
    }

    pub fn news(self) -> news::WriteCommandsAccountNewsAdmin<'a> {
        news::WriteCommandsAccountNewsAdmin::new(self.0)
    }

    pub fn permissions(self) -> permissions::WriteCommandsAccountPermissionsAdmin<'a> {
        permissions::WriteCommandsAccountPermissionsAdmin::new(self.0)
    }
}
