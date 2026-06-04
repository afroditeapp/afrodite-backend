use database_media::current::read::GetDbReadCommandsMedia;
use model::ImageProcessingDynamicConfig;
use model_media::{
    GetMediaContentFaceVerifiedNullList, MediaContentModerationQueuePage,
    MediaContentModerationQueueType, MediaContentModerationType, MediaContentType,
};
use server_common::result::Result;
use server_data::{DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead};

define_cmd_wrapper_read!(ReadCommandsMediaAdmin);

impl ReadCommandsMediaAdmin<'_> {
    pub async fn media_content_moderation_queue_page(
        &self,
        content_type: MediaContentType,
        moderation_type: MediaContentModerationType,
        queue_type: MediaContentModerationQueueType,
    ) -> Result<MediaContentModerationQueuePage, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media_admin()
                .content()
                .media_content_moderation_queue_page(content_type, moderation_type, queue_type)
        })
        .await
        .into_error()
    }

    pub async fn media_content_face_verified_null_list(
        &self,
    ) -> Result<GetMediaContentFaceVerifiedNullList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media_admin()
                .content()
                .media_content_face_verified_null_list()
        })
        .await
        .into_error()
    }

    pub async fn image_processing_config(
        &self,
    ) -> Result<Option<ImageProcessingDynamicConfig>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media_admin()
                .image_processing_config()
                .image_processing_config()
        })
        .await
        .into_error()
    }
}
