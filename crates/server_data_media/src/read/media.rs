use model_media::{
    AccountId, AccountIdInternal, ContentId, CurrentAccountMediaInternal, MediaContentRaw,
    ModerationRequest, ModerationRequestState,
};
use server_common::{
    data::{DataError, IntoDataError},
    result::Result,
};
use server_data::{define_cmd_wrapper_read, file::{utils::ContentFile, FileRead}};

use super::DbReadMedia;

define_cmd_wrapper_read!(ReadCommandsMedia);

impl ReadCommandsMedia<'_> {
    pub async fn content_data(
        &self,
        account_id: AccountId,
        content_id: ContentId,
    ) -> Result<ContentFile, DataError> {
        let c = self.files()
            .media_content(account_id, content_id);
        Ok(c)
    }

    pub async fn current_account_media(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<CurrentAccountMediaInternal, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media()
                .media_content()
                .current_account_media(account_id)
        })
        .await
        .into_error()
    }

    pub async fn all_account_media_content(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<MediaContentRaw>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media()
                .media_content()
                .get_account_media_content(account_id)
        })
        .await
        .into_error()
    }

    pub async fn moderation_request(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Option<ModerationRequest>, DataError> {
        let request = self
            .db_read(move |mut cmds| {
                cmds.media()
                    .moderation_request()
                    .moderation_request(account_id)
            })
            .await
            .into_error()?;

        if let Some(request) = request {
            let smallest_number = self
                .db_read(move |mut cmds| {
                    cmds.common()
                        .queue_number()
                        .smallest_queue_number(request.queue_type)
                })
                .await?;
            let num = if let (Some(num), true) = (
                smallest_number,
                request.state == ModerationRequestState::Waiting,
            ) {
                let queue_position = request.queue_number.0 - num;
                Some(i64::max(queue_position, 0))
            } else {
                None
            };
            Ok(Some(ModerationRequest::new(request, num)))
        } else {
            Ok(None)
        }
    }
}
