use database::{DbReadMode, DieselDatabaseError};
use database_profile::current::read::GetDbReadCommandsProfile;
use model_profile::GetMyProfileResult;
use serde::Serialize;
use server_data::data_export::SourceAccount;

#[derive(Serialize)]
pub struct UserDataExportJsonProfile {
    my_profile: GetMyProfileResult,
}

impl UserDataExportJsonProfile {
    pub fn query(
        current: &mut DbReadMode,
        id: SourceAccount,
    ) -> error_stack::Result<Self, DieselDatabaseError> {
        let id = id.0;
        let data = Self {
            my_profile: current.profile().data().my_profile(id, None)?,
        };
        Ok(data)
    }
}
