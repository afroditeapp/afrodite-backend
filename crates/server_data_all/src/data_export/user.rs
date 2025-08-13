use database::{DbReadMode, DieselDatabaseError};
use serde::Serialize;
use server_data::data_export::SourceAccount;

use crate::data_export::user::{
    account::UserDataExportJsonAccount, common::UserDataExportJsonCommon,
    media::UserDataExportJsonMedia, profile::UserDataExportJsonProfile,
};

mod account;
mod common;
mod media;
mod profile;

// TODO(future): Add news to data export. This is low priority task as
//               only admins can create or edit news.

// TODO(prod): Add more data to data export JSON

#[derive(Serialize)]
pub struct UserDataExportJson {
    common: UserDataExportJsonCommon,
    account: UserDataExportJsonAccount,
    profile: UserDataExportJsonProfile,
    pub media: UserDataExportJsonMedia,

    // Other
    note: &'static str,
}

pub fn generate_user_data_export_json(
    current: &mut DbReadMode,
    id: SourceAccount,
) -> error_stack::Result<UserDataExportJson, DieselDatabaseError> {
    let data = UserDataExportJson {
        common: UserDataExportJsonCommon::query(current, id)?,
        account: UserDataExportJsonAccount::query(current, id)?,
        profile: UserDataExportJsonProfile::query(current, id)?,
        media: UserDataExportJsonMedia::query(current, id)?,
        note: "If you created or edited news, that data is not currently included here.",
    };
    Ok(data)
}
