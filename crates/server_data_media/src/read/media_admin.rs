use database_media::current::read::GetDbReadCommandsMedia;
use model::{ImageProcessingDynamicConfig, UnixTime};
use model_media::{
    AccountId, AccountIdInternal, GetMediaContentFaceVerifiedNullList, MediaContentIdDbValue,
    MediaContentIterationPage, MediaContentIteratorPage, MediaContentIteratorPageValue,
    MediaContentIteratorSettingsInternal, MediaContentModerationQueuePage,
    MediaContentModerationQueueType, MediaContentModerationType, MediaContentRaw, MediaContentType,
};
use server_common::result::Result;
use server_data::{DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead};

define_cmd_wrapper_read!(ReadCommandsMediaAdmin);

impl ReadCommandsMediaAdmin<'_> {
    pub async fn all_account_media_content_admin(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<(MediaContentRaw, Option<AccountId>)>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media_admin()
                .content()
                .all_account_media_content_admin(account_id)
        })
        .await
        .into_error()
    }

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

    pub async fn media_content_iteration_page(
        &self,
        only_security_content: bool,
        page: i64,
        origin: UnixTime,
    ) -> Result<MediaContentIterationPage, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media_admin().content().media_content_iteration_page(
                only_security_content,
                page,
                origin,
            )
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

    pub async fn get_latest_media_content_id_db(&self) -> Result<MediaContentIdDbValue, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media_admin()
                .content()
                .get_latest_media_content_id_db()
        })
        .await
        .into_error()
    }

    pub async fn media_content_iterator_page(
        &self,
        settings: MediaContentIteratorSettingsInternal,
    ) -> Result<MediaContentIteratorPage, DataError> {
        let extra_info = settings.extra_info;
        let page = settings.page;

        let (data, has_more_pages) = self
            .db_read(move |mut cmds| {
                cmds.media_admin()
                    .content()
                    .media_content_iterator_page(settings)
            })
            .await
            .into_error()?;

        let values = data
            .into_iter()
            .map(|data| MediaContentIteratorPageValue {
                account_id: data.account_id,
                content_id: data.content_id,
                name: extra_info.name.then_some(data.name).flatten(),
                age: extra_info.age.then_some(data.age),
                upload_time: extra_info.upload_time.then_some(data.upload_time),
                secure_capture: extra_info.secure_capture.then_some(data.secure_capture),
                face_detected: extra_info.face_detected.then_some(data.face_detected),
                initial_content: extra_info.initial_content.then_some(data.initial_content),
                moderator_account_id: extra_info
                    .moderator_account_id
                    .then_some(data.moderator_account_id)
                    .flatten(),
                moderation_state: extra_info.moderation_state.then_some(data.moderation_state),
                rejected_reason_category: extra_info
                    .rejected_reason_category
                    .then_some(data.rejected_reason_category)
                    .flatten(),
                rejected_reason_details: extra_info
                    .rejected_reason_details
                    .then_some(data.rejected_reason_details)
                    .flatten(),
            })
            .collect();

        let next_page = has_more_pages.then_some(page.saturating_add(1));

        Ok(MediaContentIteratorPage { values, next_page })
    }
}
