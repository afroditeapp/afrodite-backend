use database::{DbReadMode, DieselDatabaseError};
use database_media::current::read::GetDbReadCommandsMedia;
use database_profile::current::read::GetDbReadCommandsProfile;
use model_media::ContentInfoDetailed;
use model_profile::GetMyProfileResult;
use serde::Serialize;
use server_data::data_export::SourceAccount;

// TODO(prod): Add more data to data export JSON

#[derive(Serialize)]
pub struct DataExportJson {
    my_profile: GetMyProfileResult,
    pub content: Vec<ContentInfoDetailed>,
}

pub fn generate_data_json(
    current: &mut DbReadMode,
    id: SourceAccount,
) -> error_stack::Result<DataExportJson, DieselDatabaseError> {
    let id = id.0;

    let my_profile = current.profile().data().my_profile(id, None)?;

    let internal_current_media = current
        .media()
        .media_content()
        .get_account_media_content(id)?;
    let content = internal_current_media
        .into_iter()
        .map(|m| m.into())
        .collect();

    let data = DataExportJson {
        my_profile,
        content,
    };

    Ok(data)
}
