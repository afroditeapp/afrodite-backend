use database::{DbReadMode, DieselDatabaseError};
use database_media::current::read::GetDbReadCommandsMedia;
use model_media::ContentInfoDetailed;
use serde::Serialize;
use server_data::data_export::SourceAccount;

#[derive(Serialize)]
pub struct UserDataExportJsonMedia {
    pub content: Vec<ContentInfoDetailed>,
}

impl UserDataExportJsonMedia {
    pub fn query(
        current: &mut DbReadMode,
        id: SourceAccount,
    ) -> error_stack::Result<Self, DieselDatabaseError> {
        let id = id.0;
        let data = Self {
            content: {
                let internal_current_media = current
                    .media()
                    .media_content()
                    .get_account_media_content(id)?;
                internal_current_media
                    .into_iter()
                    .map(|m| m.into())
                    .collect()
            },
        };
        Ok(data)
    }
}
