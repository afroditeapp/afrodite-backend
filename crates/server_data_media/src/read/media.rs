use database::current::read::GetDbReadCommandsCommon;
use database_media::current::read::GetDbReadCommandsMedia;
use model_media::{
    AccountId, AccountIdInternal, ContentId, CurrentAccountMediaInternal, MediaContentRaw,
    ModerationRequest, ModerationRequestState,
};
use server_common::{
    data::{DataError, IntoDataError, WrappedWithInfo},
    result::Result,
};
use server_data::{define_cmd_wrapper_read, file::{utils::ContentFile, FileRead}, read::DbRead, result::{WrappedContextExt, WrappedResultExt}};

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

    /// Check that media server has correct state for completing initial setup.
    ///
    /// Requirements:
    ///  - Account must have a moderation request.
    ///  - The current or pending security image of the account is in the request.
    ///  - The current or pending first profile image of the account is in the
    ///    request.
    pub async fn check_moderation_request_for_account(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<(), DataError> {
        let request = self
            .moderation_request(account_id)
            .await
            .change_context_with_info(DataError::Diesel, account_id)?
            .ok_or(DataError::MissingValue.report())
            .with_info(account_id)?;

        let account_media = self
            .current_account_media(account_id)
            .await
            .change_context_with_info(DataError::Diesel, account_id)?;

        // Check security content
        let current_or_pending_security_content = account_media
            .security_content_id
            .or(account_media.pending_security_content_id);
        if let Some(content) = current_or_pending_security_content {
            if !content.secure_capture {
                return Err(DataError::NotAllowed.report())
                    .with_info(account_id)
                    .with_info("Content secure capture flag is false");
            }
            if request.content.find(content.content_id()).is_none() {
                return Err(DataError::NotAllowed.report())
                    .with_info(account_id)
                    .with_info("Security content is not in moderation request");
            }
        } else {
            return Err(DataError::NotAllowed.report())
                .with_info(account_id)
                .with_info("Required security content for initial setup is not set");
        }

        // Check first profile content
        let current_or_pending_profile_content = account_media
            .profile_content_id_0
            .or(account_media.pending_profile_content_id_0);
        if let Some(content) = current_or_pending_profile_content {
            if request.content.find(content.content_id()).is_none() {
                return Err(DataError::NotAllowed.report())
                    .with_info(account_id)
                    .with_info("Content is not in moderation request");
            }
        } else {
            return Err(DataError::NotAllowed.report())
                .with_info(account_id)
                .with_info("Required content for initial setup is not set");
        }

        Ok(())
    }
}
