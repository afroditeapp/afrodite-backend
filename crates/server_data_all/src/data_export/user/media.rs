use std::net::IpAddr;

use database::{DbReadMode, DieselDatabaseError};
use database_media::current::read::GetDbReadCommandsMedia;
use model::{ContentId, ContentIdDb, ProfileContentVersion, UnixTime};
use model_chat::MediaAppNotificationSettings;
use model_media::{
    ContentInfoDetailed, MediaContentRaw, MediaStateRaw, MyProfileContent,
    SecurityContentUserDataExport,
};
use serde::Serialize;
use server_data::data_export::SourceAccount;

#[derive(Serialize)]
pub struct UserDataExportJsonMedia {
    media_state: MediaStateRaw,
    security_content: SecurityContentUserDataExport,
    profile_content_version: ProfileContentVersion,
    profile_content: MyProfileContent,
    content_extra_info: Vec<DataExportMediaContent>,
    pub content: Vec<ContentInfoDetailed>,
    media_app_notification_settings: MediaAppNotificationSettings,
}

impl UserDataExportJsonMedia {
    pub fn query(
        current: &mut DbReadMode,
        id: SourceAccount,
    ) -> simple_backend_utils::Result<Self, DieselDatabaseError> {
        let id = id.0;
        let current_media = current.media().media_content().current_account_media(id)?;
        let media_content_raw = current
            .media()
            .media_content()
            .get_account_media_content(id)?;
        let data = Self {
            media_state: current.media().get_media_state(id)?,
            security_content: SecurityContentUserDataExport::new(current_media.clone()),
            profile_content_version: current_media.profile_content_version_uuid,
            profile_content: current_media.into(),
            content_extra_info: media_content_raw
                .iter()
                .map(|m| DataExportMediaContent::new(m.clone()))
                .collect(),
            content: media_content_raw.into_iter().map(|m| m.into()).collect(),
            media_app_notification_settings: current
                .media()
                .notification()
                .app_notification_settings(id)?,
        };
        Ok(data)
    }
}

#[derive(Serialize)]
struct DataExportMediaContent {
    id: ContentIdDb,
    uuid: ContentId,
    upload_unix_time: UnixTime,
    initial_content: bool,
    source_image_pdq_hash: String,
    source_image_pdq_hash_quality: i16,
    uploader_ip: IpAddr,
    uploader_port: Option<u16>,
}

impl DataExportMediaContent {
    fn new(value: MediaContentRaw) -> Self {
        Self {
            id: value.id,
            uuid: value.uuid,
            upload_unix_time: value.upload_unix_time,
            initial_content: value.initial_content,
            source_image_pdq_hash: base16ct::lower::encode_string(&value.source_image_pdq_hash),
            source_image_pdq_hash_quality: value.source_image_pdq_hash_quality,
            uploader_ip: value.uploader_ip.to_ip_addr(),
            uploader_port: value.uploader_port.map(Into::into),
        }
    }
}
