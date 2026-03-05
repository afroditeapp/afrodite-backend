use server_data::define_cmd_wrapper_write;

mod ban;
mod client_features;
mod login;
mod news;
mod permissions;
pub use client_features::SaveInfoBannersResult;
define_cmd_wrapper_write!(WriteCommandsAccountAdmin);

impl<'a> WriteCommandsAccountAdmin<'a> {
    pub fn ban(self) -> ban::WriteCommandsAccountBan<'a> {
        ban::WriteCommandsAccountBan::new(self.0)
    }

    pub fn login(self) -> login::WriteCommandsAccountLockAdmin<'a> {
        login::WriteCommandsAccountLockAdmin::new(self.0)
    }

    pub fn client_features(self) -> client_features::WriteCommandsAccountClientFeaturesAdmin<'a> {
        client_features::WriteCommandsAccountClientFeaturesAdmin::new(self.0)
    }

    pub fn news(self) -> news::WriteCommandsAccountNewsAdmin<'a> {
        news::WriteCommandsAccountNewsAdmin::new(self.0)
    }

    pub fn permissions(self) -> permissions::WriteCommandsAccountPermissionsAdmin<'a> {
        permissions::WriteCommandsAccountPermissionsAdmin::new(self.0)
    }
}
